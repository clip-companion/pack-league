//! League Pack Daemon Entry Point
//!
//! Standalone binary that communicates with the main daemon via NDJSON over stdin/stdout.
//! Uses the gamepack-runtime crate for the protocol handling.

use std::io;
use std::sync::RwLock;

use gamepack_runtime::{
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
///
/// Uses RwLock for interior mutability so that `&self` trait methods
/// can call `&mut self` methods on the integration.
struct LeagueHandler {
    runtime: Runtime,
    integration: RwLock<LeagueIntegration>,
}

impl LeagueHandler {
    fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");
        let integration = RwLock::new(LeagueIntegration::new());
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
        let integration = self.integration.read().expect("RwLock poisoned");
        self.runtime
            .block_on(async { integration.detect_running().await })
    }

    fn get_status(&self) -> GameStatus {
        let mut integration = self.integration.write().expect("RwLock poisoned");
        let status = self.runtime.block_on(async { integration.get_status().await });

        // Convert IntegrationStatus to GameStatus
        let mut game_status = if status.connected {
            GameStatus::connected(&status.connection_status.to_string())
        } else {
            GameStatus::disconnected()
        };

        if let Some(phase) = status.game_phase {
            game_status = game_status.with_phase(phase);
        }

        game_status.in_game(status.is_in_game)
    }

    fn poll_events(&mut self) -> Vec<GameEvent> {
        let mut integration = self.integration.write().expect("RwLock poisoned");
        self.runtime
            .block_on(async { integration.poll_events().await })
    }

    fn get_live_data(&self) -> Option<serde_json::Value> {
        let mut integration = self.integration.write().expect("RwLock poisoned");
        self.runtime.block_on(async {
            integration.get_live_data().await.map(|data| data.data)
        })
    }

    fn on_session_start(&mut self) -> Option<serde_json::Value> {
        let mut integration = self.integration.write().expect("RwLock poisoned");
        self.runtime
            .block_on(async { integration.session_start().await })
    }

    fn on_session_end(&mut self, context: serde_json::Value) -> Option<MatchData> {
        let mut integration = self.integration.write().expect("RwLock poisoned");
        let result = self
            .runtime
            .block_on(async { integration.session_end(context).await });

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
        let integration = self.integration.read().expect("RwLock poisoned");
        let is_running = self.runtime.block_on(async {
            integration.detect_running().await
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

    fn get_sample_match_data(&self, subpack: u8) -> Option<serde_json::Value> {
        info!("Generating sample match data for subpack {}", subpack);
        league_integration::sample_data::generate_sample(subpack)
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
