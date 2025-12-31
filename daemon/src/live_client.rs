use serde::{Deserialize, Serialize};

use crate::{LeagueError, Result};

const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999";

pub struct LiveClientApi {
    client: reqwest::Client,
}

impl LiveClientApi {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(2))
            .build()?;

        Ok(Self { client })
    }

    pub async fn get_all_game_data(&self) -> Result<GameData> {
        let url = format!("{}/liveclientdata/allgamedata", LIVE_CLIENT_URL);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    pub async fn get_active_player(&self) -> Result<ActivePlayer> {
        let url = format!("{}/liveclientdata/activeplayer", LIVE_CLIENT_URL);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    pub async fn get_events(&self) -> Result<GameEvents> {
        let url = format!("{}/liveclientdata/eventdata", LIVE_CLIENT_URL);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Get events with both parsed and raw JSON data (for runtime discovery)
    pub async fn get_events_raw(&self) -> Result<(Vec<GameEvent>, Vec<serde_json::Value>)> {
        let url = format!("{}/liveclientdata/eventdata", LIVE_CLIENT_URL);
        let response = self.client.get(&url).send().await?;
        let json: serde_json::Value = response.json().await?;

        let events_array = json
            .get("Events")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LeagueError::ParseError("Missing Events array".to_string()))?;

        let mut events = Vec::new();
        let mut raw_events = Vec::new();

        for raw in events_array {
            // Parse structured event
            if let Ok(event) = serde_json::from_value::<GameEvent>(raw.clone()) {
                events.push(event);
                raw_events.push(raw.clone());
            }
        }

        Ok((events, raw_events))
    }

    pub async fn is_game_active(&self) -> bool {
        self.get_active_player().await.is_ok()
    }
}

impl Default for LiveClientApi {
    fn default() -> Self {
        Self::new().expect("Failed to create LiveClientApi")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub active_player: ActivePlayer,
    pub all_players: Vec<Player>,
    pub events: GameEvents,
    pub game_data: GameInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ActivePlayer {
    #[serde(default)]
    pub summoner_name: String,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub current_gold: f64,
    #[serde(default)]
    pub champion_stats: ChampionStats,
    #[serde(default)]
    pub full_runes: Option<FullRunes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FullRunes {
    #[serde(default)]
    pub keystone: Rune,
    #[serde(default)]
    pub primary_rune_tree: Rune,
    #[serde(default)]
    pub secondary_rune_tree: Rune,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Rune {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ChampionStats {
    #[serde(default)]
    pub ability_power: f64,
    #[serde(default)]
    pub armor: f64,
    #[serde(default)]
    pub attack_damage: f64,
    #[serde(default)]
    pub attack_speed: f64,
    #[serde(default)]
    pub health_regen_rate: f64,
    #[serde(default)]
    pub max_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Player {
    #[serde(default)]
    pub summoner_name: String,
    #[serde(default)]
    pub champion_name: String,
    #[serde(default)]
    pub team: String,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub scores: PlayerScores,
    #[serde(default)]
    pub is_dead: bool,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub summoner_spells: Option<SummonerSpells>,
    #[serde(default)]
    pub runes: Option<PlayerRunes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Item {
    #[serde(default, rename = "itemID")]
    pub item_id: i32,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub slot: i32,
    #[serde(default)]
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SummonerSpells {
    #[serde(default)]
    pub summoner_spell_one: SpellInfo,
    #[serde(default)]
    pub summoner_spell_two: SpellInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SpellInfo {
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerRunes {
    #[serde(default)]
    pub keystone: Rune,
    #[serde(default)]
    pub primary_rune_tree: Rune,
    #[serde(default)]
    pub secondary_rune_tree: Rune,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerScores {
    #[serde(default)]
    pub kills: i32,
    #[serde(default)]
    pub deaths: i32,
    #[serde(default)]
    pub assists: i32,
    #[serde(default)]
    pub creep_score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvents {
    #[serde(rename = "Events")]
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    #[serde(rename = "EventID")]
    pub event_id: i32,
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "EventTime")]
    pub event_time: f64,
    #[serde(rename = "KillerName", default)]
    pub killer_name: Option<String>,
    #[serde(rename = "VictimName", default)]
    pub victim_name: Option<String>,
    #[serde(rename = "Assisters", default)]
    pub assisters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub game_mode: String,
    pub game_time: f64,
    pub map_name: String,
    pub map_number: i32,
    pub map_terrain: String,
}
