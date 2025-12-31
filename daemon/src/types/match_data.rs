use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Remake,
}

impl ToString for MatchResult {
    fn to_string(&self) -> String {
        match self {
            MatchResult::Win => "win".to_string(),
            MatchResult::Loss => "loss".to_string(),
            MatchResult::Remake => "remake".to_string(),
        }
    }
}

impl From<&str> for MatchResult {
    fn from(s: &str) -> Self {
        match s {
            "win" => MatchResult::Win,
            "loss" => MatchResult::Loss,
            "remake" => MatchResult::Remake,
            _ => MatchResult::Loss,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Blue,
    Red,
}

impl ToString for Team {
    fn to_string(&self) -> String {
        match self {
            Team::Blue => "blue".to_string(),
            Team::Red => "red".to_string(),
        }
    }
}

impl From<&str> for Team {
    fn from(s: &str) -> Self {
        match s {
            "blue" => Team::Blue,
            "red" => Team::Red,
            _ => Team::Blue,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub summoner_name: String,
    pub champion: String,
    pub team: Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub id: String,
    pub game_id: i64,
    pub summoner_name: String,
    pub champion: String,
    pub champion_level: i32,
    pub result: MatchResult,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub cs_per_min: f64,
    pub vision_score: i32,
    pub kill_participation: i32,
    pub damage_dealt: i64,
    pub game_mode: String,
    pub played_at: DateTime<Utc>,
    pub duration_secs: i32,
    pub created_at: DateTime<Utc>,
    pub lp_change: Option<i32>,
    pub rank: Option<String>,
    // Summoner spells
    pub summoner_spell1: String,
    pub summoner_spell2: String,
    // Runes
    pub keystone_rune: String,
    pub secondary_tree: String,
    // Items (JSON array stored as string in DB)
    pub items: Vec<String>,
    pub trinket: Option<String>,
    // Team compositions (JSON array stored as string in DB)
    pub participants: Vec<Participant>,
    // Achievement badges (JSON array)
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchWithClips {
    #[serde(flatten)]
    pub match_data: Match,
    pub clips: Vec<Clip>,
    pub events: Vec<StoredGameEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub match_id: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub start_time_secs: f64,
    pub end_time_secs: f64,
    pub trigger_event: String,
    pub trigger_data: Option<String>,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

/// A game event stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredGameEvent {
    pub id: String,
    pub match_id: String,
    pub event_type: String,
    pub event_time_secs: f64,
    pub data: serde_json::Value,
    pub has_clip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatch {
    pub game_id: i64,
    pub summoner_name: String,
    pub champion: String,
    pub champion_level: i32,
    pub result: MatchResult,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub cs_per_min: f64,
    pub vision_score: i32,
    pub kill_participation: i32,
    pub damage_dealt: i64,
    pub game_mode: String,
    pub played_at: DateTime<Utc>,
    pub duration_secs: i32,
    pub lp_change: Option<i32>,
    pub rank: Option<String>,
    pub summoner_spell1: String,
    pub summoner_spell2: String,
    pub keystone_rune: String,
    pub secondary_tree: String,
    pub items: Vec<String>,
    pub trinket: Option<String>,
    pub participants: Vec<Participant>,
    pub badges: Vec<String>,
}
