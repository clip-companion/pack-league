//! League of Legends Integration
//!
//! This crate implements `GameIntegration` for League of Legends.
//! It provides:
//!
//! - LCU (League Client Update) API client for game state
//! - WebSocket monitoring for real-time gameflow events
//! - Live Client Data API integration for in-game stats
//! - End-of-game data collection and match finalization
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              LeagueIntegration                          │
//! │                                                         │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
//! │  │  LcuClient   │  │ LcuWebSocket │  │ LiveClient   │  │
//! │  │  (REST API)  │  │ (Gameflow)   │  │ (In-game)    │  │
//! │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
//! │         │                 │                 │          │
//! │         └─────────────────┼─────────────────┘          │
//! │                           │                            │
//! │                           ▼                            │
//! │                 GameIntegration Trait                  │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Game-Specific Database
//!
//! This integration creates and manages the `league_match_details` table
//! which stores League-specific match data (champion, KDA, items, etc.).

// Re-export the integration for daemon registration
pub use integration::LeagueIntegration;

// Public modules (types that daemon actors may need)
pub use error::*;
pub use events::*;
pub use gameflow_monitor::*;
pub use lcu::*;
pub use lcu_websocket::*;
pub use live_client::*;
pub use live_match_service::*;
pub use poller::*;
pub use state::*;
pub use triggers::*;
pub use types::*;

mod error;
mod events;
mod game_finalizer;
mod gameflow_monitor;
mod integration;
mod lcu;
mod lcu_websocket;
mod live_client;
mod live_match_service;
mod poller;
mod state;
mod triggers;
mod types;

/// League of Legends game ID (matches shared/games.json)
pub const LEAGUE_GAME_ID: i32 = 1;

/// League game slug
pub const LEAGUE_SLUG: &str = "league";
