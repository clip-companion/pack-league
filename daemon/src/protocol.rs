//! League-specific protocol types
//!
//! Re-exports shared protocol types from companion-pack-protocol and defines
//! League-specific data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-export shared protocol types
pub use companion_pack_protocol::{
    GameEvent, GameStatus, GamepackCommand, GamepackResponse, InitResponse,
    MatchData as ProtocolMatchData, PROTOCOL_VERSION,
};

// ============================================================================
// League-Specific Data Types
// ============================================================================

/// Live match data for League UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMatchData {
    /// Game ID
    pub game_id: i32,
    /// Current game time in seconds
    pub game_time_secs: f64,
    /// Game-specific live data as JSON
    pub data: Value,
}

/// Match result for League
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Remake,
}

impl std::fmt::Display for MatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchResult::Win => write!(f, "win"),
            MatchResult::Loss => write!(f, "loss"),
            MatchResult::Remake => write!(f, "remake"),
        }
    }
}

/// League-specific match data returned when a match ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchData {
    /// Game slug (e.g., "league")
    pub game_slug: String,
    /// Game ID
    pub game_id: i32,
    /// When the match was played
    pub played_at: DateTime<Utc>,
    /// Match duration in seconds
    pub duration_secs: i32,
    /// Match result
    pub result: MatchResult,
    /// Game-specific details as JSON
    pub details: Value,
}

/// Session context for tracking game session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Game-specific session data
    pub data: Value,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(data: Value) -> Self {
        Self {
            started_at: Utc::now(),
            data,
        }
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connected,
    InGame,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::InGame => write!(f, "in_game"),
        }
    }
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    /// Game slug
    pub game_slug: String,
    /// Whether connected to the game client
    pub connected: bool,
    /// Detailed connection status
    pub connection_status: ConnectionStatus,
    /// Current game phase
    pub game_phase: Option<String>,
    /// Whether actively in a match
    pub is_in_game: bool,
}

impl IntegrationStatus {
    /// Create a disconnected status
    pub fn disconnected(slug: &str) -> Self {
        Self {
            game_slug: slug.to_string(),
            connected: false,
            connection_status: ConnectionStatus::Disconnected,
            game_phase: None,
            is_in_game: false,
        }
    }
}
