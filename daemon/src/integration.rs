//! League of Legends Integration (Standalone)
//!
//! This module provides the League integration logic for the standalone gamepack.
//! It communicates with the main daemon via IPC protocol.

use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::game_finalizer::GameFinalizer;
use crate::protocol::{
    ConnectionStatus, IntegrationStatus, LiveMatchData, MatchData, MatchResult, SessionContext,
};
use crate::{GameflowPhase, LiveClientApi, LiveMatch, RankedEntry, LEAGUE_GAME_ID, LEAGUE_SLUG};

// Use shared GameEvent from the protocol crate
use companion_pack_protocol::GameEvent;

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
    /// Current game phase
    current_phase: Option<String>,
    /// Whether we're currently in game
    is_in_game: bool,
    /// Pending events to be polled
    pending_events: Vec<GameEvent>,
    /// Session context (set when session starts)
    session_context: Option<SessionContext>,
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
            current_phase: None,
            is_in_game: false,
            pending_events: Vec::new(),
            session_context: None,
        }
    }

    /// Try to get the LCU client connection
    fn try_lcu_client(&self) -> Option<crate::LcuClient> {
        crate::LcuClient::new().ok()
    }

    /// Detect if League client is running
    pub async fn detect_running(&self) -> bool {
        self.try_lcu_client().is_some()
    }

    /// Get current integration status
    pub async fn get_status(&mut self) -> IntegrationStatus {
        // Try to connect to LCU
        if let Some(client) = self.try_lcu_client() {
            self.connection_status = ConnectionStatus::Connected;

            // Get current gameflow phase
            match client.get_gameflow_phase().await {
                Ok(phase) => {
                    let is_in_game = phase.is_in_game();
                    self.current_phase = Some(phase.display_name().to_string());
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
            self.connection_status = ConnectionStatus::Disconnected;
            self.current_phase = None;
            self.is_in_game = false;
        }

        IntegrationStatus {
            game_slug: LEAGUE_SLUG.to_string(),
            connected: self.connection_status != ConnectionStatus::Disconnected,
            connection_status: self.connection_status,
            game_phase: self.current_phase.clone(),
            is_in_game: self.is_in_game,
        }
    }

    /// Poll for new game events
    pub async fn poll_events(&mut self) -> Vec<GameEvent> {
        // Drain and return pending events
        std::mem::take(&mut self.pending_events)
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

        // Capture pre-game rank for LP calculation
        self.finalizer.capture_pre_game_rank().await;

        // Get pre-game rank for context
        if let Some(client) = self.try_lcu_client() {
            if let Ok(ranks) = client.get_ranked_stats().await {
                // Get Solo/Duo queue rank
                self.pre_game_rank = ranks
                    .into_iter()
                    .find(|r| r.queue_type == "RANKED_SOLO_5x5");
                debug!("Pre-game rank: {:?}", self.pre_game_rank);
            }
        }

        // Create session context
        let context = SessionContext::new(json!({
            "pre_game_rank": self.pre_game_rank,
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

        // Reset session state
        self.session_context = None;
        *self.last_live_match.write().await = None;

        // Convert to protocol MatchData
        match_data.map(|data| {
            let result = match data.result {
                crate::MatchResult::Win => MatchResult::Win,
                crate::MatchResult::Loss => MatchResult::Loss,
                crate::MatchResult::Remake => MatchResult::Loss,
            };

            MatchData {
                game_slug: LEAGUE_SLUG.to_string(),
                game_id: LEAGUE_GAME_ID,
                played_at: Utc::now(),
                duration_secs: data.duration_secs,
                result,
                details: serde_json::to_value(&data).unwrap_or(Value::Null),
            }
        })
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
