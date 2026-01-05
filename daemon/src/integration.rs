//! League of Legends Integration (Standalone)
//!
//! This module provides the League integration logic for the standalone gamepack.
//! It communicates with the main daemon via IPC protocol.

use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::game_finalizer::GameFinalizer;
use crate::protocol::{
    ConnectionStatus, IntegrationStatus, LiveMatchData, MatchData, MatchResult, SessionContext,
};
use crate::types::GameModeContext;
use crate::{GameflowPhase, LiveClientApi, LiveMatch, RankedEntry, LEAGUE_GAME_ID, LEAGUE_SLUG};

// Use shared types from the protocol crate
use companion_pack_protocol::{emit_match_data, GameEvent, MatchDataMessage};

/// Subpack indices for League pack
pub const SUBPACK_LEAGUE: u8 = 0;
pub const SUBPACK_TFT: u8 = 1;

/// League of Legends game integration.
///
/// Monitors the League client via LCU API and provides game data
/// through the IPC protocol.
pub struct LeagueIntegration {
    /// Game finalizer for collecting end-of-game data
    finalizer: GameFinalizer,
    /// Live client API for in-game data
    live_client: Option<LiveClientApi>,
    /// Last known live match data (for session end)
    last_live_match: Arc<RwLock<Option<LiveMatch>>>,
    /// Pre-game rank for LP calculation
    pre_game_rank: Option<RankedEntry>,
    /// Current connection status
    connection_status: ConnectionStatus,
    /// Previous connection status (for change detection)
    prev_connection_status: ConnectionStatus,
    /// Current game phase
    current_phase: Option<String>,
    /// Previous game phase (for change detection)
    prev_phase: Option<String>,
    /// Whether we're currently in game
    is_in_game: bool,
    /// Pending events to be polled
    pending_events: Vec<GameEvent>,
    /// Session context (set when session starts)
    session_context: Option<SessionContext>,
    /// Last processed event ID (to avoid duplicates)
    last_event_id: i32,
    /// Current game mode context (set when session starts)
    game_mode_context: Option<GameModeContext>,
    /// Cached active player name (set when session starts, used for is_player_involved)
    active_player_name: Option<String>,
    /// Current match's external ID (game_id from LCU)
    external_match_id: Option<String>,
    /// Current subpack index (0 for League, 1 for TFT)
    current_subpack: u8,
    /// Last emitted stats (for delta detection)
    last_emitted_stats: Option<HashMap<String, serde_json::Value>>,
}

impl LeagueIntegration {
    /// Create a new League integration
    pub fn new() -> Self {
        Self {
            finalizer: GameFinalizer::new(),
            live_client: LiveClientApi::new().ok(),
            last_live_match: Arc::new(RwLock::new(None)),
            pre_game_rank: None,
            connection_status: ConnectionStatus::Disconnected,
            prev_connection_status: ConnectionStatus::Disconnected,
            current_phase: None,
            prev_phase: None,
            is_in_game: false,
            pending_events: Vec::new(),
            session_context: None,
            last_event_id: -1,
            game_mode_context: None,
            active_player_name: None,
            external_match_id: None,
            current_subpack: SUBPACK_LEAGUE,
            last_emitted_stats: None,
        }
    }

    /// Try to get the LCU client connection
    fn try_lcu_client(&self) -> Option<crate::LcuClient> {
        crate::LcuClient::new().ok()
    }

    /// Get current subpack index based on game mode
    pub fn current_subpack(&self) -> u8 {
        self.current_subpack
    }

    /// Check if currently playing TFT
    pub fn is_tft(&self) -> bool {
        self.game_mode_context
            .as_ref()
            .map(|c| c.is_tft())
            .unwrap_or(false)
    }

    /// Detect if League client is running
    pub async fn detect_running(&self) -> bool {
        self.try_lcu_client().is_some()
    }

    /// Get current integration status
    pub async fn get_status(&mut self) -> IntegrationStatus {
        // Try to connect to LCU
        if let Some(client) = self.try_lcu_client() {
            let new_status = ConnectionStatus::Connected;

            // Emit ClientConnected event if status changed from Disconnected
            if self.prev_connection_status == ConnectionStatus::Disconnected
                && new_status != ConnectionStatus::Disconnected
            {
                info!("LCU client connected");
                self.pending_events.push(GameEvent::new(
                    "ClientConnected".to_string(),
                    0.0,
                    json!({}),
                ));
            }

            self.connection_status = new_status;

            // Get current gameflow phase
            match client.get_gameflow_phase().await {
                Ok(phase) => {
                    let is_in_game = phase.is_in_game();
                    let new_phase = Some(phase.display_name().to_string());

                    // Emit PhaseChanged event if phase changed
                    if self.prev_phase != new_phase {
                        info!(
                            "Gameflow phase changed: {:?} -> {:?}",
                            self.prev_phase, new_phase
                        );
                        self.pending_events.push(GameEvent::new(
                            "PhaseChanged".to_string(),
                            0.0,
                            json!({
                                "from": self.prev_phase,
                                "to": new_phase,
                                "phase": phase.display_name(),
                            }),
                        ));
                        self.prev_phase = new_phase.clone();
                    }

                    self.current_phase = new_phase;
                    self.is_in_game = is_in_game;

                    if is_in_game {
                        self.connection_status = ConnectionStatus::InGame;
                    }
                }
                Err(e) => {
                    debug!("Failed to get gameflow phase: {}", e);
                }
            }
        } else {
            // Emit ClientDisconnected event if status changed to Disconnected
            if self.prev_connection_status != ConnectionStatus::Disconnected
                && self.connection_status != ConnectionStatus::Disconnected
            {
                info!("LCU client disconnected");
                self.pending_events.push(GameEvent::new(
                    "ClientDisconnected".to_string(),
                    0.0,
                    json!({}),
                ));
            }

            self.connection_status = ConnectionStatus::Disconnected;
            self.current_phase = None;
            self.prev_phase = None;
            self.is_in_game = false;
        }

        // Update previous status for next comparison
        self.prev_connection_status = self.connection_status;

        IntegrationStatus {
            game_slug: LEAGUE_SLUG.to_string(),
            connected: self.connection_status != ConnectionStatus::Disconnected,
            connection_status: self.connection_status,
            game_phase: self.current_phase.clone(),
            is_in_game: self.is_in_game,
        }
    }

    /// Poll for new game events from the Live Client Data API
    pub async fn poll_events(&mut self) -> Vec<GameEvent> {
        // Check LCU status first - this emits ClientConnected/Disconnected/PhaseChanged events
        let _ = self.get_status().await;

        let mut events = std::mem::take(&mut self.pending_events);

        // Only poll if we have a live client and are in game
        if let Some(ref live_client) = self.live_client {
            // Try to get events from the Live Client API
            match live_client.get_events().await {
                Ok(game_events) => {
                    // Use cached player name, or try to fetch it if not cached
                    let player_name = if let Some(ref name) = self.active_player_name {
                        name.clone()
                    } else {
                        // Try to fetch and cache the player name
                        match live_client.get_active_player().await {
                            Ok(player) => {
                                info!("Cached active player name: {}", player.summoner_name);
                                self.active_player_name = Some(player.summoner_name.clone());
                                player.summoner_name
                            }
                            Err(e) => {
                                debug!("Failed to get active player: {}", e);
                                String::new()
                            }
                        }
                    };

                    for event in game_events.events {
                        // Skip already processed events
                        if event.event_id <= self.last_event_id {
                            continue;
                        }
                        self.last_event_id = event.event_id;

                        // Check if player is involved in this event (only if we have a valid player name)
                        let is_player_involved = !player_name.is_empty() && (
                            event.killer_name.as_ref() == Some(&player_name)
                            || event.victim_name.as_ref() == Some(&player_name)
                            || event.assisters.contains(&player_name)
                        );

                        // Create game event using protocol types
                        let game_event = GameEvent::new(
                            event.event_name.clone(),
                            event.event_time,
                            serde_json::json!({
                                "event_id": event.event_id,
                                "killer_name": event.killer_name,
                                "victim_name": event.victim_name,
                                "assisters": event.assisters,
                                "is_player_involved": is_player_involved,
                            }),
                        );

                        info!(
                            "Game event: {} at {:.1}s (player_involved: {})",
                            event.event_name, event.event_time, is_player_involved
                        );

                        events.push(game_event);
                    }
                }
                Err(e) => {
                    // Only log at debug level - game might not be active
                    debug!("Failed to poll events: {}", e);
                }
            }
        }

        events
    }

    /// Get live match data
    pub async fn get_live_data(&mut self) -> Option<LiveMatchData> {
        if !self.is_in_game {
            return None;
        }

        // Try to get live data from live client API
        if let Some(ref live_client) = self.live_client {
            match live_client.get_all_game_data().await {
                Ok(game_data) => {
                    if let Some(live_match) = LiveMatch::from_game_data(&game_data) {
                        // Store for session end
                        *self.last_live_match.write().await = Some(live_match.clone());

                        return Some(LiveMatchData {
                            game_id: LEAGUE_GAME_ID,
                            game_time_secs: live_match.game_time_secs,
                            data: serde_json::to_value(&live_match).unwrap_or(Value::Null),
                        });
                    }
                }
                Err(e) => {
                    debug!("Failed to get live match data: {}", e);
                }
            }
        }

        None
    }

    /// Start a game session
    pub async fn session_start(&mut self) -> Option<Value> {
        info!("League session starting");

        // Reset event tracking for new session
        self.last_event_id = -1;
        self.is_in_game = true;
        self.active_player_name = None;
        self.external_match_id = None;
        self.current_subpack = SUBPACK_LEAGUE;
        self.last_emitted_stats = None;

        // Try to pre-fetch active player name from Live Client API
        if let Some(ref live_client) = self.live_client {
            if let Ok(player) = live_client.get_active_player().await {
                info!("Active player name: {}", player.summoner_name);
                self.active_player_name = Some(player.summoner_name);
            }
        }

        // Capture pre-game rank for LP calculation
        self.finalizer.capture_pre_game_rank().await;

        // Get pre-game rank and game mode context
        if let Some(client) = self.try_lcu_client() {
            // Get game mode from gameflow session first (needed to determine which rank to fetch)
            if let Ok(session) = client.get_gameflow_session().await {
                let game_mode = session.game_mode();
                let queue = &session.game_data.queue;

                // Store external match ID from game data
                let game_id = session.game_data.game_id;
                if game_id != 0 {
                    self.external_match_id = Some(game_id.to_string());
                    info!("Match external ID: {}", game_id);
                }

                self.game_mode_context = Some(GameModeContext::from_session(
                    game_mode,
                    queue.id,
                    &queue.name,
                    queue.is_ranked,
                ));

                // Determine subpack based on game mode
                let is_tft = session.is_tft();
                self.current_subpack = if is_tft { SUBPACK_TFT } else { SUBPACK_LEAGUE };

                info!(
                    "Game mode detected: {} (queue: {}, ranked: {}, subpack: {})",
                    self.game_mode_context.as_ref().map(|c| c.display_name.as_str()).unwrap_or("unknown"),
                    queue.name,
                    queue.is_ranked,
                    self.current_subpack
                );
            }

            // Get ranked stats - select appropriate queue based on game mode
            if let Ok(ranks) = client.get_ranked_stats().await {
                let is_tft = self.game_mode_context.as_ref().map(|c| c.is_tft()).unwrap_or(false);

                self.pre_game_rank = if is_tft {
                    // TFT uses RANKED_TFT, RANKED_TFT_DOUBLE_UP, or RANKED_TFT_TURBO
                    ranks.into_iter().find(|r| r.queue_type.starts_with("RANKED_TFT"))
                } else {
                    // Regular League uses RANKED_SOLO_5x5 or RANKED_FLEX_SR
                    ranks.into_iter().find(|r| r.queue_type == "RANKED_SOLO_5x5")
                };

                debug!("Pre-game rank: {:?}", self.pre_game_rank);
            }
        }

        // Create session context with game mode info
        let context = SessionContext::new(json!({
            "pre_game_rank": self.pre_game_rank,
            "game_mode": self.game_mode_context,
            "subpack": self.current_subpack,
            "external_match_id": self.external_match_id,
        }));

        self.session_context = Some(context.clone());

        Some(serde_json::to_value(&context).unwrap_or(Value::Null))
    }

    /// End a game session and return match data
    pub async fn session_end(&mut self, _context: Value) -> Option<MatchData> {
        info!("League session ending");

        // Get the last live match data
        let last_match = self.last_live_match.read().await.clone();

        // Get post-game data from finalizer
        let match_data = self.finalizer.finalize_game(last_match).await.ok().flatten();

        // Capture values before resetting
        let game_mode_ctx = self.game_mode_context.take();
        let subpack = self.current_subpack;
        let external_match_id = self.external_match_id.take();

        // Reset session state
        self.session_context = None;
        self.active_player_name = None;
        self.last_emitted_stats = None;
        *self.last_live_match.write().await = None;

        // If we have an external match ID, emit SetComplete to the daemon
        if let Some(ref external_id) = external_match_id {
            // Build final stats from the match data
            let final_stats = match_data.as_ref().map(|data| {
                self.build_stats_map(data, &game_mode_ctx)
            });

            // Emit SetComplete message
            let summary_source = if match_data.is_some() { "api" } else { "live_fallback" };
            emit_match_data(MatchDataMessage::SetComplete {
                subpack,
                external_match_id: external_id.clone(),
                summary_source: summary_source.to_string(),
                final_stats,
            });

            info!(
                "Emitted SetComplete for match {} (subpack: {}, source: {})",
                external_id, subpack, summary_source
            );
        }

        // Convert to protocol MatchData (for backwards compat)
        match_data.map(|data| {
            let result = match data.result {
                crate::MatchResult::Win => MatchResult::Win,
                crate::MatchResult::Loss => MatchResult::Loss,
                crate::MatchResult::Remake => MatchResult::Loss,
            };

            // Include game mode in details
            let mut details = serde_json::to_value(&data).unwrap_or(Value::Null);
            if let Some(ref mode_ctx) = game_mode_ctx {
                if let Value::Object(ref mut map) = details {
                    map.insert("game_mode".to_string(), serde_json::to_value(mode_ctx).unwrap_or(Value::Null));
                }
            }

            MatchData {
                game_slug: LEAGUE_SLUG.to_string(),
                game_id: LEAGUE_GAME_ID,
                played_at: Utc::now(),
                duration_secs: data.duration_secs,
                result,
                details,
            }
        })
    }

    /// Build a stats HashMap from match data for the current subpack
    fn build_stats_map(
        &self,
        data: &crate::CreateMatch,
        game_mode_ctx: &Option<GameModeContext>,
    ) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // Common fields for both League and TFT
        stats.insert("summoner_name".to_string(), json!(data.summoner_name));
        stats.insert("game_mode".to_string(), json!(data.game_mode));
        stats.insert("game_id".to_string(), json!(data.game_id));

        if self.current_subpack == SUBPACK_LEAGUE {
            // League-specific stats
            stats.insert("champion".to_string(), json!(data.champion));
            stats.insert("champion_level".to_string(), json!(data.champion_level));
            stats.insert("kills".to_string(), json!(data.kills));
            stats.insert("deaths".to_string(), json!(data.deaths));
            stats.insert("assists".to_string(), json!(data.assists));
            stats.insert("cs".to_string(), json!(data.cs));
            stats.insert("cs_per_min".to_string(), json!(data.cs_per_min));
            stats.insert("vision_score".to_string(), json!(data.vision_score));
            stats.insert("kill_participation".to_string(), json!(data.kill_participation));
            stats.insert("damage_dealt".to_string(), json!(data.damage_dealt));
            stats.insert("summoner_spell1".to_string(), json!(data.summoner_spell1));
            stats.insert("summoner_spell2".to_string(), json!(data.summoner_spell2));
            stats.insert("keystone_rune".to_string(), json!(data.keystone_rune));
            stats.insert("secondary_tree".to_string(), json!(data.secondary_tree));
            stats.insert("items_json".to_string(), json!(data.items));
            stats.insert("trinket".to_string(), json!(data.trinket));
            stats.insert("participants_json".to_string(), json!(data.participants));
            stats.insert("badges_json".to_string(), json!(data.badges));
        }
        // TFT stats would be different - to be implemented when TFT support is added

        if let Some(ref mode_ctx) = game_mode_ctx {
            stats.insert("queue_type".to_string(), json!(mode_ctx.queue_name));
        }

        if let Some(lp) = data.lp_change {
            stats.insert("lp_change".to_string(), json!(lp));
        }
        if let Some(ref rank) = data.rank {
            stats.insert("rank".to_string(), json!(rank));
        }

        stats
    }

    /// Add a game event
    pub fn add_event(&mut self, event: GameEvent) {
        self.pending_events.push(event);
    }
}

impl Default for LeagueIntegration {
    fn default() -> Self {
        Self::new()
    }
}
