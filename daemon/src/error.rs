//! Error types for League integration

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LeagueError>;

#[derive(Debug, Error)]
pub enum LeagueError {
    #[error("LCU not found: {0}")]
    LcuNotFound(String),

    #[error("LCU connection failed: {0}")]
    LcuConnectionFailed(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("League of Legends is not running")]
    LeagueNotRunning,

    #[error("{0}")]
    Other(String),
}

// Alias for compatibility with code that uses AppError
pub type AppError = LeagueError;
