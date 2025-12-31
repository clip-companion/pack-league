//! League of Legends client state tracking
//!
//! This module provides comprehensive state detection for the League client,
//! determining whether the user is in the client, queue, champion select, or in-game.

use serde::{Deserialize, Serialize};

/// The current phase of the League of Legends client/game.
///
/// These phases are returned by the LCU API at `/lol-gameflow/v1/gameflow-phase`.
/// We use this to determine how to handle video capture and compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum GameflowPhase {
    /// Client is not running or phase is unknown
    #[default]
    None,
    /// In the main lobby (pre-queue)
    Lobby,
    /// Searching for a match
    Matchmaking,
    /// Ready check popup appeared
    ReadyCheck,
    /// In champion select
    ChampSelect,
    /// Game is starting/loading
    GameStart,
    /// Failed to launch the game
    FailedToLaunch,
    /// Actively in a game
    InProgress,
    /// Reconnecting to a game
    Reconnect,
    /// Waiting for end-of-game stats
    WaitingForStats,
    /// Pre end-of-game screen
    PreEndOfGame,
    /// End of game lobby (victory/defeat screen)
    EndOfGame,
    /// Game terminated with an error
    TerminatedInError,
    /// Checked into a tournament
    CheckedIntoTournament,
}

impl GameflowPhase {
    /// Returns true if the user is actively in a game
    pub fn is_in_game(&self) -> bool {
        matches!(
            self,
            GameflowPhase::InProgress | GameflowPhase::Reconnect | GameflowPhase::GameStart
        )
    }

    /// Returns true if the user is in the client (not in-game)
    pub fn is_in_client(&self) -> bool {
        matches!(
            self,
            GameflowPhase::Lobby
                | GameflowPhase::Matchmaking
                | GameflowPhase::ReadyCheck
                | GameflowPhase::ChampSelect
                | GameflowPhase::WaitingForStats
                | GameflowPhase::PreEndOfGame
                | GameflowPhase::EndOfGame
        )
    }

    /// Returns true if we should be recording
    pub fn should_record(&self) -> bool {
        // Record during gameplay and champion select
        matches!(
            self,
            GameflowPhase::ChampSelect
                | GameflowPhase::GameStart
                | GameflowPhase::InProgress
                | GameflowPhase::Reconnect
        )
    }

    /// Returns the display name for this phase
    pub fn display_name(&self) -> &'static str {
        match self {
            GameflowPhase::None => "Not Running",
            GameflowPhase::Lobby => "In Lobby",
            GameflowPhase::Matchmaking => "Finding Match",
            GameflowPhase::ReadyCheck => "Match Found",
            GameflowPhase::ChampSelect => "Champion Select",
            GameflowPhase::GameStart => "Game Starting",
            GameflowPhase::FailedToLaunch => "Failed to Launch",
            GameflowPhase::InProgress => "In Game",
            GameflowPhase::Reconnect => "Reconnecting",
            GameflowPhase::WaitingForStats => "Loading Stats",
            GameflowPhase::PreEndOfGame => "Game Ending",
            GameflowPhase::EndOfGame => "Post Game",
            GameflowPhase::TerminatedInError => "Error",
            GameflowPhase::CheckedIntoTournament => "Tournament",
        }
    }
}

impl From<&str> for GameflowPhase {
    fn from(s: &str) -> Self {
        match s {
            "None" => GameflowPhase::None,
            "Lobby" => GameflowPhase::Lobby,
            "Matchmaking" => GameflowPhase::Matchmaking,
            "ReadyCheck" => GameflowPhase::ReadyCheck,
            "ChampSelect" => GameflowPhase::ChampSelect,
            "GameStart" => GameflowPhase::GameStart,
            "FailedToLaunch" => GameflowPhase::FailedToLaunch,
            "InProgress" => GameflowPhase::InProgress,
            "Reconnect" => GameflowPhase::Reconnect,
            "WaitingForStats" => GameflowPhase::WaitingForStats,
            "PreEndOfGame" => GameflowPhase::PreEndOfGame,
            "EndOfGame" => GameflowPhase::EndOfGame,
            "TerminatedInError" => GameflowPhase::TerminatedInError,
            "CheckedIntoTournament" => GameflowPhase::CheckedIntoTournament,
            _ => GameflowPhase::None,
        }
    }
}

impl std::fmt::Display for GameflowPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Capture mode determined by the current game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Not capturing anything
    Idle,
    /// Capturing the client window (smaller, needs staging)
    ClientWindow,
    /// Capturing the full game (scales to fill)
    FullscreenGame,
}

impl CaptureMode {
    /// Determine capture mode from gameflow phase
    pub fn from_phase(phase: GameflowPhase) -> Self {
        match phase {
            GameflowPhase::InProgress | GameflowPhase::Reconnect => CaptureMode::FullscreenGame,
            GameflowPhase::ChampSelect => CaptureMode::ClientWindow,
            _ => CaptureMode::Idle,
        }
    }

    /// Returns true if the captured content should be centered on a stage
    /// with a background (for smaller client windows)
    pub fn needs_staging(&self) -> bool {
        matches!(self, CaptureMode::ClientWindow)
    }

    /// Returns true if the captured content should scale to fill the output
    pub fn scale_to_fill(&self) -> bool {
        matches!(self, CaptureMode::FullscreenGame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gameflow_phase_is_in_game() {
        assert!(GameflowPhase::InProgress.is_in_game());
        assert!(GameflowPhase::Reconnect.is_in_game());
        assert!(GameflowPhase::GameStart.is_in_game());
        assert!(!GameflowPhase::ChampSelect.is_in_game());
        assert!(!GameflowPhase::Lobby.is_in_game());
    }

    #[test]
    fn test_gameflow_phase_is_in_client() {
        assert!(GameflowPhase::Lobby.is_in_client());
        assert!(GameflowPhase::ChampSelect.is_in_client());
        assert!(GameflowPhase::EndOfGame.is_in_client());
        assert!(!GameflowPhase::InProgress.is_in_client());
        assert!(!GameflowPhase::None.is_in_client());
    }

    #[test]
    fn test_gameflow_phase_should_record() {
        assert!(GameflowPhase::InProgress.should_record());
        assert!(GameflowPhase::ChampSelect.should_record());
        assert!(!GameflowPhase::Lobby.should_record());
        assert!(!GameflowPhase::EndOfGame.should_record());
    }

    #[test]
    fn test_gameflow_phase_from_str() {
        assert_eq!(GameflowPhase::from("InProgress"), GameflowPhase::InProgress);
        assert_eq!(GameflowPhase::from("ChampSelect"), GameflowPhase::ChampSelect);
        assert_eq!(GameflowPhase::from("Unknown"), GameflowPhase::None);
    }

    #[test]
    fn test_capture_mode_from_phase() {
        assert_eq!(
            CaptureMode::from_phase(GameflowPhase::InProgress),
            CaptureMode::FullscreenGame
        );
        assert_eq!(
            CaptureMode::from_phase(GameflowPhase::ChampSelect),
            CaptureMode::ClientWindow
        );
        assert_eq!(
            CaptureMode::from_phase(GameflowPhase::Lobby),
            CaptureMode::Idle
        );
    }

    #[test]
    fn test_capture_mode_staging() {
        assert!(CaptureMode::ClientWindow.needs_staging());
        assert!(!CaptureMode::FullscreenGame.needs_staging());
        assert!(!CaptureMode::Idle.needs_staging());
    }

    #[test]
    fn test_capture_mode_scale_to_fill() {
        assert!(CaptureMode::FullscreenGame.scale_to_fill());
        assert!(!CaptureMode::ClientWindow.scale_to_fill());
        assert!(!CaptureMode::Idle.scale_to_fill());
    }
}
