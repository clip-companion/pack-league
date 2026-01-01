//! Game mode definitions with GUID-based identification
//!
//! This allows consistent identification of game modes across the system,
//! similar to how pack events use GUIDs.

use serde::{Deserialize, Serialize};

/// A game mode with GUID-based identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GameMode {
    /// Unique identifier for this game mode
    pub guid: &'static str,
    /// Internal key from the game API (e.g., "CLASSIC", "TFT", "ARAM")
    pub api_key: &'static str,
    /// Human-readable display name
    pub display_name: &'static str,
    /// Whether this mode uses placement (1st-8th) instead of win/loss
    pub is_placement_based: bool,
    /// Whether this mode has traditional KDA stats
    pub has_kda: bool,
    /// Whether this mode is a team game (vs free-for-all like TFT)
    pub is_team_based: bool,
}

// ============================================================================
// Known Game Modes (from League client)
// ============================================================================

/// Classic Summoner's Rift (5v5)
pub const CLASSIC: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440001",
    api_key: "CLASSIC",
    display_name: "Summoner's Rift",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// Teamfight Tactics
pub const TFT: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440002",
    api_key: "TFT",
    display_name: "Teamfight Tactics",
    is_placement_based: true,
    has_kda: false,
    is_team_based: false,
};

/// ARAM (All Random All Mid)
pub const ARAM: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440003",
    api_key: "ARAM",
    display_name: "ARAM",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// Arena (2v2v2v2)
pub const ARENA: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440004",
    api_key: "CHERRY",
    display_name: "Arena",
    is_placement_based: true,  // Placement 1st-4th
    has_kda: true,
    is_team_based: true,  // 2-player teams
};

/// URF (Ultra Rapid Fire)
pub const URF: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440005",
    api_key: "URF",
    display_name: "URF",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// One for All
pub const ONE_FOR_ALL: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440006",
    api_key: "ONEFORALL",
    display_name: "One for All",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// Nexus Blitz
pub const NEXUS_BLITZ: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440007",
    api_key: "NEXUSBLITZ",
    display_name: "Nexus Blitz",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// Unknown/Other game mode (fallback)
pub const UNKNOWN: GameMode = GameMode {
    guid: "gm-550e8400-0001-4a00-a716-446655440000",
    api_key: "UNKNOWN",
    display_name: "Unknown Mode",
    is_placement_based: false,
    has_kda: true,
    is_team_based: true,
};

/// All known game modes
pub const ALL_MODES: &[&GameMode] = &[
    &CLASSIC,
    &TFT,
    &ARAM,
    &ARENA,
    &URF,
    &ONE_FOR_ALL,
    &NEXUS_BLITZ,
];

/// Look up a game mode by its API key (case-insensitive)
pub fn from_api_key(key: &str) -> &'static GameMode {
    let key_upper = key.to_uppercase();
    for mode in ALL_MODES {
        if mode.api_key == key_upper {
            return mode;
        }
    }
    // Check for alternative keys
    match key_upper.as_str() {
        "PRACTICETOOL" => &CLASSIC,
        "TUTORIAL" => &CLASSIC,
        "CHERRY" => &ARENA,
        _ => &UNKNOWN,
    }
}

/// Look up a game mode by its GUID
pub fn from_guid(guid: &str) -> Option<&'static GameMode> {
    ALL_MODES.iter().find(|m| m.guid == guid).copied()
}

/// Info stored in session/match context about the current game mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameModeContext {
    /// GUID of the game mode
    pub mode_guid: String,
    /// API key for reference
    pub mode_key: String,
    /// Display name
    pub display_name: String,
    /// Queue ID from the game client
    pub queue_id: i32,
    /// Queue name (e.g., "Ranked Solo/Duo", "TFT Ranked")
    pub queue_name: String,
    /// Whether this is a ranked game
    pub is_ranked: bool,
}

impl GameModeContext {
    /// Create context from LCU session data
    pub fn from_session(game_mode: &str, queue_id: i32, queue_name: &str, is_ranked: bool) -> Self {
        let mode = from_api_key(game_mode);
        Self {
            mode_guid: mode.guid.to_string(),
            mode_key: mode.api_key.to_string(),
            display_name: mode.display_name.to_string(),
            queue_id,
            queue_name: queue_name.to_string(),
            is_ranked,
        }
    }

    /// Check if this is a TFT game
    pub fn is_tft(&self) -> bool {
        self.mode_guid == TFT.guid
    }

    /// Get the game mode definition
    pub fn game_mode(&self) -> &'static GameMode {
        from_guid(&self.mode_guid).unwrap_or(&UNKNOWN)
    }
}
