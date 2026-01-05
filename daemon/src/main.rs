//! League Pack Daemon Entry Point
//!
//! Standalone binary that communicates with the main daemon via NDJSON over stdin/stdout.
//! Uses the shared companion-pack-protocol crate for the protocol handling.

use std::io;

use companion_pack_protocol::{
    run_gamepack, GameEvent, GameStatus, GamepackHandler, GamepackResult, InitResponse,
    IsMatchInProgressResponse, MatchData,
};
use tokio::runtime::Runtime;
use tracing::info;
use tracing_subscriber::EnvFilter;

use league_integration::LeagueIntegration;

/// Game ID for League of Legends
const GAME_ID: i32 = 1;

/// Game slug
const SLUG: &str = "league";

/// Wrapper that implements GamepackHandler for LeagueIntegration
struct LeagueHandler {
    runtime: Runtime,
    integration: LeagueIntegration,
}

impl LeagueHandler {
    fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");
        let integration = LeagueIntegration::new();
        Self {
            runtime,
            integration,
        }
    }
}

impl GamepackHandler for LeagueHandler {
    fn init(&mut self) -> GamepackResult<InitResponse> {
        info!("Initializing League integration");
        Ok(InitResponse {
            game_id: GAME_ID,
            slug: SLUG.to_string(),
            protocol_version: companion_pack_protocol::PROTOCOL_VERSION,
        })
    }

    fn detect_running(&self) -> bool {
        self.runtime
            .block_on(async { self.integration.detect_running().await })
    }

    fn get_status(&self) -> GameStatus {
        // We need &mut self for get_status, so we use interior mutability pattern
        // For now, return a basic status - the integration will be refactored later
        let running = self.detect_running();
        if running {
            GameStatus::connected("League client detected")
        } else {
            GameStatus::disconnected()
        }
    }

    fn poll_events(&mut self) -> Vec<GameEvent> {
        self.runtime
            .block_on(async { self.integration.poll_events().await })
    }

    fn get_live_data(&self) -> Option<serde_json::Value> {
        // Would need &mut self for the full implementation
        // For now return None - will be properly implemented with async refactor
        None
    }

    fn on_session_start(&mut self) -> Option<serde_json::Value> {
        self.runtime
            .block_on(async { self.integration.session_start().await })
    }

    fn on_session_end(&mut self, context: serde_json::Value) -> Option<MatchData> {
        let result = self
            .runtime
            .block_on(async { self.integration.session_end(context).await });

        // Convert from local MatchData to protocol MatchData
        result.map(|m| MatchData::new(m.game_slug, m.game_id, m.result.to_string(), m.details))
    }

    fn shutdown(&mut self) {
        info!("League pack shutting down");
    }

    fn is_match_in_progress(
        &self,
        subpack: u8,
        external_match_id: &str,
    ) -> IsMatchInProgressResponse {
        // Check if the game is actually still running
        let is_running = self.runtime.block_on(async {
            self.integration.detect_running().await
        });

        if !is_running {
            info!(
                "Match {} (subpack {}) not in progress - game not running",
                external_match_id, subpack
            );
            // Game isn't running, so the match is definitely not in progress
            // We could try to fetch final stats from Riot API here, but for now
            // just return that it ended
            IsMatchInProgressResponse::ended()
        } else {
            // Game is running - the match may still be in progress
            // The integration's is_in_game would be more accurate but requires state
            info!(
                "Match {} (subpack {}) may still be in progress - game running",
                external_match_id, subpack
            );
            IsMatchInProgressResponse::still_playing()
        }
    }
}

fn main() {
    // Initialize logging to stderr (stdout is reserved for protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("pack_league=debug".parse().unwrap()),
        )
        .with_writer(io::stderr)
        .init();

    info!(
        "League pack daemon starting (protocol v{})",
        companion_pack_protocol::PROTOCOL_VERSION
    );

    // Create handler and run the main loop
    let handler = LeagueHandler::new();
    run_gamepack(handler);

    info!("League pack daemon shut down");
}
