use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum LeagueEventType {
    GameStart,
    GameEnd,
    ChampionKill,
    Multikill,
    Ace,
    FirstBlood,
    TurretKilled,
    InhibKilled,
    DragonKill,
    HeraldKill,
    BaronKill,
    InhibRespawningSoon,
    InhibRespawned,
    #[serde(other)]
    Unknown,
}

impl From<&str> for LeagueEventType {
    fn from(s: &str) -> Self {
        match s {
            "GameStart" => LeagueEventType::GameStart,
            "GameEnd" => LeagueEventType::GameEnd,
            "ChampionKill" => LeagueEventType::ChampionKill,
            "Multikill" => LeagueEventType::Multikill,
            "Ace" => LeagueEventType::Ace,
            "FirstBlood" => LeagueEventType::FirstBlood,
            "TurretKilled" => LeagueEventType::TurretKilled,
            "InhibKilled" => LeagueEventType::InhibKilled,
            "DragonKill" => LeagueEventType::DragonKill,
            "HeraldKill" => LeagueEventType::HeraldKill,
            "BaronKill" => LeagueEventType::BaronKill,
            "InhibRespawningSoon" => LeagueEventType::InhibRespawningSoon,
            "InhibRespawned" => LeagueEventType::InhibRespawned,
            _ => LeagueEventType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedGameEvent {
    pub event_type: LeagueEventType,
    pub event_time: f64,
    pub killer_name: Option<String>,
    pub victim_name: Option<String>,
    pub assisters: Vec<String>,
    pub is_player_involved: bool,
}
