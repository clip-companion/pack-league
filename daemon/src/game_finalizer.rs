use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};

use crate::{CreateMatch, LiveMatch, MatchResult, Participant, Team};
use crate::{EndOfGameStats, LcuClient, LocalPlayerStats, RankedEntry};

/// Convert summoner spell ID to name
fn spell_id_to_name(id: i32) -> String {
    match id {
        1 => "Cleanse",
        3 => "Exhaust",
        4 => "Flash",
        6 => "Ghost",
        7 => "Heal",
        11 => "Smite",
        12 => "Teleport",
        13 => "Clarity",
        14 => "Ignite",
        21 => "Barrier",
        32 => "Snowball",
        _ => return format!("{}", id),
    }
    .to_string()
}

/// Convert keystone rune ID to name
fn keystone_id_to_name(id: i32) -> String {
    match id {
        // Precision
        8005 => "Press the Attack",
        8008 => "Lethal Tempo",
        8021 => "Fleet Footwork",
        8010 => "Conqueror",
        // Domination
        8112 => "Electrocute",
        8124 => "Predator",
        8128 => "Dark Harvest",
        9923 => "Hail of Blades",
        // Sorcery
        8214 => "Summon Aery",
        8229 => "Arcane Comet",
        8230 => "Phase Rush",
        // Resolve
        8437 => "Grasp of the Undying",
        8439 => "Aftershock",
        8465 => "Guardian",
        // Inspiration
        8351 => "Glacial Augment",
        8360 => "Unsealed Spellbook",
        8369 => "First Strike",
        _ => return format!("{}", id),
    }
    .to_string()
}

/// Convert rune tree ID to name
fn rune_tree_id_to_name(id: i32) -> String {
    match id {
        8000 => "Precision",
        8100 => "Domination",
        8200 => "Sorcery",
        8300 => "Inspiration",
        8400 => "Resolve",
        _ => return format!("{}", id),
    }
    .to_string()
}

/// Service that finalizes game data when a match ends and saves it to the database
pub struct GameFinalizer {
    pre_game_rank: Option<RankedEntry>,
}

impl GameFinalizer {
    pub fn new() -> Self {
        Self {
            pre_game_rank: None,
        }
    }

    /// Store the player's rank at the start of the game for LP calculation
    pub async fn capture_pre_game_rank(&mut self) {
        if let Ok(lcu) = LcuClient::new() {
            if let Ok(ranks) = lcu.get_ranked_stats().await {
                // Get Solo/Duo queue rank (RANKED_SOLO_5x5)
                self.pre_game_rank = ranks
                    .into_iter()
                    .find(|r| r.queue_type == "RANKED_SOLO_5x5");

                if let Some(ref rank) = self.pre_game_rank {
                    info!(
                        "Captured pre-game rank: {} {} ({}LP)",
                        rank.tier, rank.division, rank.league_points
                    );
                }
            }
        }
    }

    /// Finalize the game and return match data for saving
    /// Note: The caller (daemon actor) is responsible for saving to database
    pub async fn finalize_game(
        &mut self,
        last_live_match: Option<LiveMatch>,
    ) -> Result<Option<CreateMatch>> {
        info!("Finalizing game...");

        // Try to get end of game stats from LCU
        let eog_stats = match LcuClient::new() {
            Ok(lcu) => match lcu.get_end_of_game_stats().await {
                Ok(stats) => Some(stats),
                Err(e) => {
                    warn!("Failed to get end of game stats: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to connect to LCU: {}", e);
                None
            }
        };

        // Get post-game rank for LP calculation
        let post_game_rank = if let Ok(lcu) = LcuClient::new() {
            lcu.get_ranked_stats()
                .await
                .ok()
                .and_then(|ranks| ranks.into_iter().find(|r| r.queue_type == "RANKED_SOLO_5x5"))
        } else {
            None
        };

        // Calculate LP change
        let lp_change = match (&self.pre_game_rank, &post_game_rank) {
            (Some(pre), Some(post)) => {
                // Simple LP diff - doesn't account for promotion/demotion
                Some(post.league_points - pre.league_points)
            }
            _ => None,
        };

        // Get current rank string
        let rank_str = post_game_rank
            .as_ref()
            .map(|r| format!("{} {}", r.tier, r.division));

        // Create match record from available data
        let create_match = if let Some(eog) = eog_stats {
            self.create_match_from_eog(eog, lp_change, rank_str)
        } else if let Some(live) = last_live_match {
            self.create_match_from_live(live, lp_change, rank_str)
        } else {
            warn!("No game data available to finalize");
            return Ok(None);
        };

        // Clear pre-game rank
        self.pre_game_rank = None;

        Ok(create_match)
    }

    /// Create match from end-of-game stats (most complete data)
    fn create_match_from_eog(
        &self,
        eog: EndOfGameStats,
        lp_change: Option<i32>,
        rank: Option<String>,
    ) -> Option<CreateMatch> {
        let local = eog.local_player.as_ref()?;
        let stats = &local.stats;

        // Determine win/loss
        let result = if stats.win {
            MatchResult::Win
        } else {
            MatchResult::Loss
        };

        // Calculate total CS
        let total_cs = stats.minions_killed + stats.neutral_minions_killed;
        let game_mins = eog.game_length as f64 / 60.0;
        let cs_per_min = if game_mins > 0.0 {
            total_cs as f64 / game_mins
        } else {
            0.0
        };

        // Calculate kill participation
        let team_kills: i32 = eog
            .teams
            .iter()
            .find(|t| t.team_id == local.team_id)
            .map(|t| t.players.iter().map(|p| p.stats.champions_killed).sum())
            .unwrap_or(0);

        let kill_participation = if team_kills > 0 {
            ((stats.champions_killed + stats.assists) as f64 / team_kills as f64 * 100.0) as i32
        } else {
            0
        };

        // Build participants list
        let participants: Vec<Participant> = eog
            .teams
            .iter()
            .flat_map(|t| {
                let team = if t.team_id == 100 { Team::Blue } else { Team::Red };
                t.players.iter().map(move |p| Participant {
                    summoner_name: p.summoner_name.clone(),
                    champion: p.champion_name.clone(),
                    team: team.clone(),
                })
            })
            .collect();

        // Compute badges from stats
        let badges = self.compute_badges(local, &eog);

        Some(CreateMatch {
            game_id: eog.game_id,
            summoner_name: local.summoner_name.clone(),
            champion: local.champion_name.clone(),
            champion_level: stats.level,
            result,
            kills: stats.champions_killed,
            deaths: stats.num_deaths,
            assists: stats.assists,
            cs: total_cs,
            cs_per_min,
            vision_score: stats.vision_score,
            kill_participation,
            damage_dealt: stats.total_damage_dealt_to_champions,
            game_mode: eog.game_mode.clone(),
            played_at: Utc::now(),
            duration_secs: eog.game_length,
            lp_change,
            rank,
            summoner_spell1: spell_id_to_name(local.spell1_id),
            summoner_spell2: spell_id_to_name(local.spell2_id),
            keystone_rune: keystone_id_to_name(local.perk0),
            secondary_tree: rune_tree_id_to_name(local.perk_sub_style),
            items: local.items.iter().take(6).map(|i| format!("{}", i)).collect(),
            trinket: local.items.get(6).map(|i| format!("{}", i)),
            participants,
            badges,
        })
    }

    /// Create match from live match data (fallback when EOG not available)
    fn create_match_from_live(
        &self,
        live: LiveMatch,
        lp_change: Option<i32>,
        rank: Option<String>,
    ) -> Option<CreateMatch> {
        // We can't determine win/loss from live data alone
        // Default to loss as a conservative estimate
        let result = MatchResult::Loss;

        let game_mins = live.game_time_secs / 60.0;
        let cs_per_min = if game_mins > 0.0 {
            live.cs as f64 / game_mins
        } else {
            0.0
        };

        // Calculate kill participation from live participants
        let team_kills: i32 = live
            .participants
            .iter()
            .filter(|p| p.team == live.team)
            .map(|p| p.kills)
            .sum();

        let kill_participation = if team_kills > 0 {
            ((live.kills + live.assists) as f64 / team_kills as f64 * 100.0) as i32
        } else {
            0
        };

        let participants: Vec<Participant> = live
            .participants
            .iter()
            .map(|p| Participant {
                summoner_name: p.summoner_name.clone(),
                champion: p.champion.clone(),
                team: p.team.clone(),
            })
            .collect();

        Some(CreateMatch {
            game_id: 0, // Unknown from live data
            summoner_name: live.summoner_name,
            champion: live.champion,
            champion_level: live.level,
            result,
            kills: live.kills,
            deaths: live.deaths,
            assists: live.assists,
            cs: live.cs,
            cs_per_min,
            vision_score: 0, // Not available from live data
            kill_participation,
            damage_dealt: 0, // Not available from live data
            game_mode: live.game_mode,
            played_at: Utc::now(),
            duration_secs: live.game_time_secs as i32,
            lp_change,
            rank,
            summoner_spell1: live.spell1.map(|s| s.name).unwrap_or_default(),
            summoner_spell2: live.spell2.map(|s| s.name).unwrap_or_default(),
            keystone_rune: live.runes.as_ref().map(|r| r.keystone_name.clone()).unwrap_or_default(),
            secondary_tree: live.runes.as_ref().map(|r| r.secondary_tree_name.clone()).unwrap_or_default(),
            items: live.items.iter().map(|i| i.name.clone()).collect(),
            trinket: live.trinket.map(|t| t.name),
            participants,
            badges: vec![],
        })
    }

    /// Compute achievement badges from end of game stats
    fn compute_badges(&self, local: &LocalPlayerStats, eog: &EndOfGameStats) -> Vec<String> {
        let mut badges = Vec::new();
        let stats = &local.stats;

        // Perfect game (no deaths)
        if stats.num_deaths == 0 && (stats.champions_killed > 0 || stats.assists > 0) {
            badges.push("Perfect".to_string());
        }

        // Legendary KDA (5+ KDA)
        let kda = if stats.num_deaths > 0 {
            (stats.champions_killed + stats.assists) as f64 / stats.num_deaths as f64
        } else {
            (stats.champions_killed + stats.assists) as f64
        };

        if kda >= 5.0 && stats.num_deaths > 0 {
            badges.push("Legendary".to_string());
        }

        // MVP candidate (most kills on winning team)
        if stats.win {
            let team = eog.teams.iter().find(|t| t.team_id == local.team_id);
            if let Some(t) = team {
                let max_kills = t.players.iter().map(|p| p.stats.champions_killed).max().unwrap_or(0);
                if stats.champions_killed == max_kills && max_kills > 0 {
                    badges.push("MVP".to_string());
                }
            }
        }

        // High CS
        let game_mins = eog.game_length as f64 / 60.0;
        let total_cs = stats.minions_killed + stats.neutral_minions_killed;
        if game_mins > 0.0 && total_cs as f64 / game_mins >= 8.0 {
            badges.push("Farm Master".to_string());
        }

        badges
    }
}

impl Default for GameFinalizer {
    fn default() -> Self {
        Self::new()
    }
}
