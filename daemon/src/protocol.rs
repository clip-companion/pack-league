//! Gamepack IPC Protocol Types
//!
//! Defines the NDJSON messages exchanged between the main daemon and this gamepack.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Commands (Daemon → Gamepack)
// ============================================================================

/// Commands sent from the main daemon to this gamepack
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamepackCommand {
    /// Initialize the integration
    Init { request_id: String },

    /// Check if the game is running
    DetectRunning { request_id: String },

    /// Get current game status
    GetStatus { request_id: String },

    /// Poll for new events
    PollEvents { request_id: String },

    /// Get live match data
    GetLiveData { request_id: String },

    /// Session started - capture any pre-game data
    SessionStart { request_id: String },

    /// Session ended - return match data
    SessionEnd {
        request_id: String,
        context: Value, // Opaque context from SessionStart
    },

    /// Shutdown gracefully
    Shutdown { request_id: String },
}

// ============================================================================
// Responses (Gamepack → Daemon)
// ============================================================================

/// Responses sent from this gamepack to the main daemon
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamepackResponse {
    /// Initialization complete
    Initialized {
        request_id: String,
        game_id: i32,
        slug: String,
        protocol_version: u32,
    },

    /// Game running status
    RunningStatus { request_id: String, running: bool },

    /// Current game status
    GameStatus {
        request_id: String,
        connected: bool,
        connection_status: String,
        game_phase: Option<String>,
        is_in_game: bool,
    },

    /// Polled events
    Events {
        request_id: String,
        events: Vec<GameEvent>,
    },

    /// Live match data
    LiveData {
        request_id: String,
        data: Option<LiveMatchData>,
    },

    /// Session started with context
    SessionStarted {
        request_id: String,
        context: Option<Value>,
    },

    /// Session ended with match data
    SessionEnded {
        request_id: String,
        match_data: Option<MatchData>,
    },

    /// Error response
    Error {
        request_id: String,
        message: String,
        code: Option<String>,
    },

    /// Shutdown complete
    ShutdownComplete { request_id: String },
}

// ============================================================================
// Data Types
// ============================================================================

/// A game event that can trigger clip recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// Event type (e.g., "ChampionKill", "DragonKill")
    pub event_type: String,
    /// Timestamp within the game session (seconds)
    pub timestamp_secs: f64,
    /// Event-specific data as JSON
    pub data: Value,
    /// Suggested clip timing (seconds before event)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_capture_secs: Option<f64>,
    /// Suggested clip timing (seconds after event)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_capture_secs: Option<f64>,
}

impl GameEvent {
    /// Create a new game event
    pub fn new(event_type: impl Into<String>, timestamp_secs: f64) -> Self {
        Self {
            event_type: event_type.into(),
            timestamp_secs,
            data: Value::Null,
            pre_capture_secs: None,
            post_capture_secs: None,
        }
    }

    /// Add data to the event
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Set clip capture timing
    pub fn with_timing(mut self, pre_secs: f64, post_secs: f64) -> Self {
        self.pre_capture_secs = Some(pre_secs);
        self.post_capture_secs = Some(post_secs);
        self
    }
}

/// Live match data for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMatchData {
    /// Game ID
    pub game_id: i32,
    /// Current game time in seconds
    pub game_time_secs: f64,
    /// Game-specific live data as JSON
    pub data: Value,
}

/// Match result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

impl std::fmt::Display for MatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchResult::Win => write!(f, "win"),
            MatchResult::Loss => write!(f, "loss"),
            MatchResult::Draw => write!(f, "draw"),
        }
    }
}

/// Data returned when a match ends
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
