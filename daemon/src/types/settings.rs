//! League-specific settings types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerSettings {
    pub on_kill: bool,
    pub on_death: bool,
    pub on_assist: bool,
    pub on_multikill: bool,
    pub on_tower_kill: bool,
    pub on_dragon: bool,
    pub on_baron: bool,
    pub on_ace: bool,
}

impl Default for TriggerSettings {
    fn default() -> Self {
        Self {
            on_kill: true,
            on_death: true,
            on_assist: false,
            on_multikill: true,
            on_tower_kill: false,
            on_dragon: true,
            on_baron: true,
            on_ace: true,
        }
    }
}
