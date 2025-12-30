//! League of Legends GameIntegration Implementation
//!
//! This module implements the `GameIntegration` trait for League of Legends.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use league_companion_api::{
    ConnectionStatus, GameEvent as ApiGameEvent, GameIntegration, GenericResult,
    IntegrationStatus, LiveMatchData, MatchData, SessionContext,
};
use serde_json::{json, Value};
use tokio_rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::game_finalizer::GameFinalizer;
use crate::{LiveClientApi, LiveMatch, RankedEntry, LEAGUE_GAME_ID, LEAGUE_SLUG};

/// SQL migration for the league_match_details table
const LEAGUE_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS league_match_details (
    match_id TEXT PRIMARY KEY REFERENCES matches(id) ON DELETE CASCADE,

    -- Result (game-specific - League has remake, others might not)
    result TEXT NOT NULL CHECK(result IN ('win', 'loss', 'remake')),

    -- Player identity
    summoner_name TEXT NOT NULL,

    -- Champion
    champion TEXT NOT NULL,
    champion_level INTEGER NOT NULL DEFAULT 1,

    -- Performance stats
    kills INTEGER NOT NULL DEFAULT 0,
    deaths INTEGER NOT NULL DEFAULT 0,
    assists INTEGER NOT NULL DEFAULT 0,
    cs INTEGER NOT NULL DEFAULT 0,
    cs_per_min REAL NOT NULL DEFAULT 0.0,
    vision_score INTEGER NOT NULL DEFAULT 0,
    kill_participation INTEGER NOT NULL DEFAULT 0,
    damage_dealt INTEGER NOT NULL DEFAULT 0,

    -- Game context
    game_mode TEXT NOT NULL,
    lp_change INTEGER,
    rank TEXT,

    -- Loadout
    summoner_spell1 TEXT NOT NULL DEFAULT 'Flash',
    summoner_spell2 TEXT NOT NULL DEFAULT 'Ignite',
    keystone_rune TEXT NOT NULL DEFAULT 'Electrocute',
    secondary_tree TEXT NOT NULL DEFAULT 'Sorcery',
    items TEXT NOT NULL DEFAULT '[]',
    trinket TEXT,

    -- Team data
    participants TEXT NOT NULL DEFAULT '[]',
    badges TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_league_champion
    ON league_match_details(champion);
"#;

/// League of Legends game integration.
///
/// Monitors the League client via LCU API and WebSocket,
/// and collects match data when games end.
pub struct LeagueIntegration {
    /// Game finalizer for collecting end-of-game data
    finalizer: GameFinalizer,
    /// Live client API for in-game data
    live_client: Option<LiveClientApi>,
    /// Last known live match data (for session end)
    last_live_match: Arc<RwLock<Option<LiveMatch>>>,
    /// Pre-game rank for LP calculation
    pre_game_rank: Option<RankedEntry>,
}

impl LeagueIntegration {
    /// Create a new League integration
    pub fn new() -> Self {
        Self {
            finalizer: GameFinalizer::new(),
            live_client: LiveClientApi::new().ok(),
            last_live_match: Arc::new(RwLock::new(None)),
            pre_game_rank: None,
        }
    }

    /// Try to get the LCU client connection
    fn try_lcu_client(&self) -> Option<crate::LcuClient> {
        crate::LcuClient::new().ok()
    }
}

impl Default for LeagueIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameIntegration for LeagueIntegration {
    fn slug(&self) -> &'static str {
        LEAGUE_SLUG
    }

    fn game_id(&self) -> i32 {
        LEAGUE_GAME_ID
    }

    async fn detect_running(&self) -> bool {
        // Try to connect to the LCU - if we can, League client is running
        self.try_lcu_client().is_some()
    }

    async fn get_status(&self) -> IntegrationStatus {
        match self.try_lcu_client() {
            Some(client) => {
                // Try to get gameflow phase to determine if in game
                match client.get_gameflow_phase().await {
                    Ok(phase) => {
                        let is_in_game = phase.is_in_game();
                        let connection_status = if is_in_game {
                            ConnectionStatus::InGame
                        } else {
                            ConnectionStatus::Connected
                        };
                        IntegrationStatus {
                            game_slug: LEAGUE_SLUG.to_string(),
                            connected: true,
                            connection_status,
                            game_phase: Some(format!("{:?}", phase)),
                            is_in_game,
                        }
                    }
                    Err(_) => IntegrationStatus {
                        game_slug: LEAGUE_SLUG.to_string(),
                        connected: true,
                        connection_status: ConnectionStatus::Connected,
                        game_phase: None,
                        is_in_game: false,
                    },
                }
            }
            None => IntegrationStatus::disconnected(LEAGUE_SLUG),
        }
    }

    async fn on_session_start(&mut self) -> Option<SessionContext> {
        tracing::info!("League session starting - capturing pre-game rank");

        // Capture pre-game rank for LP calculation
        self.finalizer.capture_pre_game_rank().await;

        // Initialize live client if needed
        if self.live_client.is_none() {
            self.live_client = LiveClientApi::new().ok();
        }

        // Clear last live match data
        *self.last_live_match.write().await = None;

        // Return session context with start time
        Some(SessionContext::new(json!({
            "started_at": Utc::now().to_rfc3339(),
        })))
    }

    async fn on_session_end(&mut self, _ctx: SessionContext) -> Option<MatchData> {
        tracing::info!("League session ending - collecting match data");

        // Get last live match data
        let last_live = self.last_live_match.read().await.clone();

        // Finalize game and get match data
        match self.finalizer.finalize_game(last_live).await {
            Ok(Some(create_match)) => {
                // Convert League-specific result to generic result
                use crate::MatchResult;
                let result = match create_match.result {
                    MatchResult::Win => GenericResult::Win,
                    MatchResult::Loss => GenericResult::Loss,
                    MatchResult::Remake => GenericResult::Draw, // Treat remake as draw
                };

                // Convert to JSON for details
                let details = json!({
                    "result": create_match.result.to_string(),
                    "summonerName": create_match.summoner_name,
                    "champion": create_match.champion,
                    "championLevel": create_match.champion_level,
                    "kills": create_match.kills,
                    "deaths": create_match.deaths,
                    "assists": create_match.assists,
                    "cs": create_match.cs,
                    "csPerMin": create_match.cs_per_min,
                    "visionScore": create_match.vision_score,
                    "killParticipation": create_match.kill_participation,
                    "damageDealt": create_match.damage_dealt,
                    "gameMode": create_match.game_mode,
                    "lpChange": create_match.lp_change,
                    "rank": create_match.rank,
                    "summonerSpell1": create_match.summoner_spell1,
                    "summonerSpell2": create_match.summoner_spell2,
                    "keystoneRune": create_match.keystone_rune,
                    "secondaryTree": create_match.secondary_tree,
                    "items": create_match.items,
                    "trinket": create_match.trinket,
                    "participants": create_match.participants,
                    "badges": create_match.badges,
                });

                Some(MatchData {
                    game_slug: LEAGUE_SLUG.to_string(),
                    game_id: LEAGUE_GAME_ID,
                    played_at: create_match.played_at,
                    duration_secs: create_match.duration_secs,
                    result,
                    details,
                })
            }
            Ok(None) => {
                tracing::warn!("No match data available");
                None
            }
            Err(e) => {
                tracing::error!("Failed to finalize game: {}", e);
                None
            }
        }
    }

    async fn poll_events(&self) -> Vec<ApiGameEvent> {
        // Get events from Live Client API
        let Some(client) = &self.live_client else {
            return vec![];
        };

        match client.get_events_raw().await {
            Ok((events, raw_events)) => {
                // Also update last live match from full game data
                if let Ok(game_data) = client.get_all_game_data().await {
                    if let Some(live_match) = LiveMatch::from_game_data(&game_data) {
                        *self.last_live_match.write().await = Some(live_match);
                    }
                }

                // Convert ALL game events to API events (no filtering)
                // Include raw JSON data for runtime discovery
                events
                    .iter()
                    .zip(raw_events.iter())
                    .map(|(event, raw)| {
                        ApiGameEvent::new(&event.event_name, event.event_time)
                            .with_data(raw.clone())
                            .with_timing(10.0, 5.0)
                    })
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    async fn get_live_data(&self) -> Option<LiveMatchData> {
        let Some(client) = &self.live_client else {
            return None;
        };

        match client.get_all_game_data().await {
            Ok(game_data) => {
                let live_match = LiveMatch::from_game_data(&game_data)?;

                // Update cached live match
                *self.last_live_match.write().await = Some(live_match.clone());

                Some(LiveMatchData {
                    game_id: LEAGUE_GAME_ID,
                    game_time_secs: live_match.game_time_secs,
                    data: serde_json::to_value(&live_match).ok()?,
                })
            }
            Err(_) => None,
        }
    }

    fn migrations(&self) -> &'static [&'static str] {
        &[LEAGUE_MIGRATION]
    }

    async fn save_match_details(
        &self,
        conn: &Connection,
        match_id: &str,
        details: Value,
    ) -> Result<()> {
        let match_id = match_id.to_string();

        conn.call(move |conn| {
            // Parse the JSON details into individual fields
            let result = details.get("result").and_then(|v| v.as_str()).unwrap_or("loss");
            let summoner_name = details.get("summonerName").and_then(|v| v.as_str()).unwrap_or("");
            let champion = details.get("champion").and_then(|v| v.as_str()).unwrap_or("");
            let champion_level = details.get("championLevel").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            let kills = details.get("kills").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let deaths = details.get("deaths").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let assists = details.get("assists").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let cs = details.get("cs").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let cs_per_min = details.get("csPerMin").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let vision_score = details.get("visionScore").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let kill_participation = details.get("killParticipation").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let damage_dealt = details.get("damageDealt").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let game_mode = details.get("gameMode").and_then(|v| v.as_str()).unwrap_or("");
            let lp_change: Option<i32> = details.get("lpChange").and_then(|v| v.as_i64()).map(|v| v as i32);
            let rank = details.get("rank").and_then(|v| v.as_str()).map(|s| s.to_string());
            let summoner_spell1 = details.get("summonerSpell1").and_then(|v| v.as_str()).unwrap_or("Flash");
            let summoner_spell2 = details.get("summonerSpell2").and_then(|v| v.as_str()).unwrap_or("Ignite");
            let keystone_rune = details.get("keystoneRune").and_then(|v| v.as_str()).unwrap_or("Electrocute");
            let secondary_tree = details.get("secondaryTree").and_then(|v| v.as_str()).unwrap_or("Sorcery");
            let items = details.get("items").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
            let trinket = details.get("trinket").and_then(|v| v.as_str()).map(|s| s.to_string());
            let participants = details.get("participants").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
            let badges = details.get("badges").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());

            conn.execute(
                "INSERT OR REPLACE INTO league_match_details (
                    match_id, result, summoner_name, champion, champion_level,
                    kills, deaths, assists, cs, cs_per_min, vision_score,
                    kill_participation, damage_dealt, game_mode, lp_change, rank,
                    summoner_spell1, summoner_spell2, keystone_rune, secondary_tree,
                    items, trinket, participants, badges
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                    ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
                )",
                tokio_rusqlite::rusqlite::params![
                    match_id, result, summoner_name, champion, champion_level,
                    kills, deaths, assists, cs, cs_per_min, vision_score,
                    kill_participation, damage_dealt, game_mode, lp_change, rank,
                    summoner_spell1, summoner_spell2, keystone_rune, secondary_tree,
                    items, trinket, participants, badges
                ],
            )?;

            Ok::<(), tokio_rusqlite::rusqlite::Error>(())
        })
        .await?;

        Ok(())
    }

    async fn get_match_details(&self, conn: &Connection, match_id: &str) -> Result<Option<Value>> {
        let match_id = match_id.to_string();

        let result = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT
                        result, summoner_name, champion, champion_level,
                        kills, deaths, assists, cs, cs_per_min, vision_score,
                        kill_participation, damage_dealt, game_mode, lp_change, rank,
                        summoner_spell1, summoner_spell2, keystone_rune, secondary_tree,
                        items, trinket, participants, badges
                    FROM league_match_details
                    WHERE match_id = ?1",
                )?;

                let row = stmt.query_row([&match_id], |row| {
                    Ok(serde_json::json!({
                        "result": row.get::<_, String>(0)?,
                        "summonerName": row.get::<_, String>(1)?,
                        "champion": row.get::<_, String>(2)?,
                        "championLevel": row.get::<_, i32>(3)?,
                        "kills": row.get::<_, i32>(4)?,
                        "deaths": row.get::<_, i32>(5)?,
                        "assists": row.get::<_, i32>(6)?,
                        "cs": row.get::<_, i32>(7)?,
                        "csPerMin": row.get::<_, f64>(8)?,
                        "visionScore": row.get::<_, i32>(9)?,
                        "killParticipation": row.get::<_, i32>(10)?,
                        "damageDealt": row.get::<_, i32>(11)?,
                        "gameMode": row.get::<_, String>(12)?,
                        "lpChange": row.get::<_, Option<i32>>(13)?,
                        "rank": row.get::<_, Option<String>>(14)?,
                        "summonerSpell1": row.get::<_, String>(15)?,
                        "summonerSpell2": row.get::<_, String>(16)?,
                        "keystoneRune": row.get::<_, String>(17)?,
                        "secondaryTree": row.get::<_, String>(18)?,
                        "items": serde_json::from_str::<Value>(&row.get::<_, String>(19)?).unwrap_or(Value::Array(vec![])),
                        "trinket": row.get::<_, Option<String>>(20)?,
                        "participants": serde_json::from_str::<Value>(&row.get::<_, String>(21)?).unwrap_or(Value::Array(vec![])),
                        "badges": serde_json::from_str::<Value>(&row.get::<_, String>(22)?).unwrap_or(Value::Array(vec![])),
                    }))
                });

                match row {
                    Ok(details) => Ok::<Option<Value>, tokio_rusqlite::rusqlite::Error>(Some(details)),
                    Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })
            .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_identity() {
        let integration = LeagueIntegration::new();
        assert_eq!(integration.slug(), "league");
        assert_eq!(integration.game_id(), 1);
    }

    #[test]
    fn test_migrations() {
        let integration = LeagueIntegration::new();
        let migrations = integration.migrations();
        assert_eq!(migrations.len(), 1);
        assert!(migrations[0].contains("league_match_details"));
    }
}
