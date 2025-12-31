use serde::{Deserialize, Serialize};

use super::Team;

/// Item in a slot (0-5 are regular items, 6 is trinket)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveItem {
    pub item_id: i32,
    pub name: String,
    pub slot: i32,
}

/// Summoner spell info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSpell {
    pub name: String,
}

/// Rune info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRunes {
    pub keystone_id: i32,
    pub keystone_name: String,
    pub primary_tree_id: i32,
    pub primary_tree_name: String,
    pub secondary_tree_id: i32,
    pub secondary_tree_name: String,
}

/// Represents a player in an active game with real-time stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
    pub summoner_name: String,
    pub champion: String,
    pub team: Team,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub level: i32,
    pub is_dead: bool,
}

/// Represents the current game state with real-time data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveMatch {
    pub summoner_name: String,
    pub champion: String,
    pub level: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub current_gold: f64,
    pub game_time_secs: f64,
    pub game_mode: String,
    pub team: Team,
    /// Items in slots 0-5 (regular items)
    pub items: Vec<LiveItem>,
    /// Trinket in slot 6
    pub trinket: Option<LiveItem>,
    /// Summoner spells
    pub spell1: Option<LiveSpell>,
    pub spell2: Option<LiveSpell>,
    /// Rune info
    pub runes: Option<LiveRunes>,
    pub participants: Vec<LivePlayer>,
    pub is_dead: bool,
}

impl LiveMatch {
    /// Creates a LiveMatch from Live Client API GameData
    pub fn from_game_data(game_data: &crate::GameData) -> Option<Self> {
        let active_player = &game_data.active_player;
        let game_info = &game_data.game_data;

        // Find the active player in the all_players list to get their team and scores
        let player = game_data
            .all_players
            .iter()
            .find(|p| p.summoner_name == active_player.summoner_name)?;

        let team = match player.team.to_lowercase().as_str() {
            "order" | "blue" => Team::Blue,
            "chaos" | "red" => Team::Red,
            _ => Team::Blue,
        };

        // Extract items (slots 0-5) and trinket (slot 6)
        let mut items: Vec<LiveItem> = Vec::new();
        let mut trinket: Option<LiveItem> = None;

        for item in &player.items {
            let live_item = LiveItem {
                item_id: item.item_id,
                name: item.display_name.clone(),
                slot: item.slot,
            };
            if item.slot == 6 {
                trinket = Some(live_item);
            } else {
                items.push(live_item);
            }
        }

        // Extract summoner spells
        let (spell1, spell2) = if let Some(ref spells) = player.summoner_spells {
            (
                Some(LiveSpell {
                    name: spells.summoner_spell_one.display_name.clone(),
                }),
                Some(LiveSpell {
                    name: spells.summoner_spell_two.display_name.clone(),
                }),
            )
        } else {
            (None, None)
        };

        // Extract runes
        let runes = player.runes.as_ref().map(|r| LiveRunes {
            keystone_id: r.keystone.id,
            keystone_name: r.keystone.display_name.clone(),
            primary_tree_id: r.primary_rune_tree.id,
            primary_tree_name: r.primary_rune_tree.display_name.clone(),
            secondary_tree_id: r.secondary_rune_tree.id,
            secondary_tree_name: r.secondary_rune_tree.display_name.clone(),
        });

        let participants: Vec<LivePlayer> = game_data
            .all_players
            .iter()
            .map(|p| {
                let player_team = match p.team.to_lowercase().as_str() {
                    "order" | "blue" => Team::Blue,
                    "chaos" | "red" => Team::Red,
                    _ => Team::Blue,
                };

                LivePlayer {
                    summoner_name: p.summoner_name.clone(),
                    champion: p.champion_name.clone(),
                    team: player_team,
                    kills: p.scores.kills,
                    deaths: p.scores.deaths,
                    assists: p.scores.assists,
                    cs: p.scores.creep_score,
                    level: p.level,
                    is_dead: p.is_dead,
                }
            })
            .collect();

        Some(LiveMatch {
            summoner_name: active_player.summoner_name.clone(),
            champion: player.champion_name.clone(),
            level: active_player.level,
            kills: player.scores.kills,
            deaths: player.scores.deaths,
            assists: player.scores.assists,
            cs: player.scores.creep_score,
            current_gold: active_player.current_gold,
            game_time_secs: game_info.game_time,
            game_mode: game_info.game_mode.clone(),
            team,
            items,
            trinket,
            spell1,
            spell2,
            runes,
            participants,
            is_dead: player.is_dead,
        })
    }
}
