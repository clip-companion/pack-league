//! Sample match data generation for UI preview/testing.
//!
//! Generates randomized but valid match data that can be used to preview
//! the MatchCard component without requiring actual game data.

use chrono::{Duration, Utc};
use rand::prelude::*;
use serde_json::{json, Value};

use crate::integration::{SUBPACK_LEAGUE, SUBPACK_TFT};

/// Popular champions for sample data (subset for variety)
const CHAMPIONS: &[&str] = &[
    "Aatrox", "Ahri", "Akali", "Akshan", "Alistar", "Amumu", "Anivia", "Annie",
    "Aphelios", "Ashe", "AurelionSol", "Azir", "Bard", "Blitzcrank", "Brand",
    "Braum", "Caitlyn", "Camille", "Cassiopeia", "Darius", "Diana", "Draven",
    "Ekko", "Elise", "Evelynn", "Ezreal", "Fiora", "Fizz", "Galio", "Garen",
    "Gnar", "Gragas", "Graves", "Gwen", "Hecarim", "Heimerdinger", "Irelia",
    "Ivern", "Janna", "JarvanIV", "Jax", "Jayce", "Jhin", "Jinx", "Kaisa",
    "Karma", "Kassadin", "Katarina", "Kayle", "Kayn", "Kennen", "Khazix",
    "Kindred", "Kled", "KogMaw", "Leblanc", "LeeSin", "Leona", "Lillia",
    "Lissandra", "Lucian", "Lulu", "Lux", "Malphite", "Malzahar", "Maokai",
    "MasterYi", "MissFortune", "Mordekaiser", "Morgana", "Nami", "Nasus",
    "Nautilus", "Neeko", "Nidalee", "Nocturne", "Nunu", "Olaf", "Orianna",
    "Ornn", "Pantheon", "Poppy", "Pyke", "Qiyana", "Quinn", "Rakan", "Rammus",
    "RekSai", "Rell", "Renata", "Renekton", "Rengar", "Riven", "Rumble",
    "Ryze", "Samira", "Sejuani", "Senna", "Seraphine", "Sett", "Shaco",
    "Shen", "Shyvana", "Singed", "Sion", "Sivir", "Skarner", "Sona", "Soraka",
    "Swain", "Sylas", "Syndra", "TahmKench", "Taliyah", "Talon", "Taric",
    "Teemo", "Thresh", "Tristana", "Trundle", "Tryndamere", "TwistedFate",
    "Twitch", "Udyr", "Urgot", "Varus", "Vayne", "Veigar", "Velkoz", "Vex",
    "Vi", "Viego", "Viktor", "Vladimir", "Volibear", "Warwick", "Wukong",
    "Xayah", "Xerath", "XinZhao", "Yasuo", "Yone", "Yorick", "Yuumi", "Zac",
    "Zed", "Zeri", "Ziggs", "Zilean", "Zoe", "Zyra",
];

/// Summoner spells
const SPELLS: &[&str] = &[
    "SummonerFlash", "SummonerTeleport", "SummonerIgnite", "SummonerHeal",
    "SummonerBarrier", "SummonerExhaust", "SummonerCleanse", "SummonerGhost",
    "SummonerSmite",
];

/// Keystone runes
const KEYSTONES: &[&str] = &[
    "Electrocute", "DarkHarvest", "HailOfBlades", "Predator",
    "Conqueror", "FleetFootwork", "LethalTempo", "PressTheAttack",
    "SummonAery", "ArcaneComet", "PhaseRush",
    "GraspOfTheUndying", "Aftershock", "Guardian",
    "FirstStrike", "GlacialAugment", "UnsealedSpellbook",
];

/// Rune trees (for secondary)
const RUNE_TREES: &[&str] = &[
    "Domination", "Precision", "Sorcery", "Resolve", "Inspiration",
];

/// Item names (sample set)
const ITEMS: &[&str] = &[
    "Infinity Edge", "Kraken Slayer", "Galeforce", "Shieldbow",
    "Divine Sunderer", "Trinity Force", "Stridebreaker",
    "Luden's Tempest", "Liandry's Anguish", "Everfrost", "Crown of the Shattered Queen",
    "Eclipse", "Duskblade of Draktharr", "Prowler's Claw",
    "Riftmaker", "Night Harvester", "Hextech Rocketbelt",
    "Goredrinker", "Sunfire Aegis", "Frostfire Gauntlet",
    "Turbo Chemtank", "Jak'Sho, The Protean", "Heartsteel",
    "Immortal Shieldbow", "Navori Quickblades", "The Collector",
    "Lord Dominik's Regards", "Mortal Reminder", "Rapid Firecannon",
    "Phantom Dancer", "Runaan's Hurricane", "Blade of the Ruined King",
    "Wit's End", "Guinsoo's Rageblade", "Nashor's Tooth",
    "Rabadon's Deathcap", "Void Staff", "Shadowflame",
    "Horizon Focus", "Cosmic Drive", "Mejai's Soulstealer",
    "Zhonya's Hourglass", "Banshee's Veil", "Morellonomicon",
    "Demonic Embrace", "Rylai's Crystal Scepter",
    "Dead Man's Plate", "Force of Nature", "Thornmail",
    "Randuin's Omen", "Gargoyle Stoneplate", "Warmog's Armor",
    "Spirit Visage", "Anathema's Chains",
    "Redemption", "Mikael's Blessing", "Shurelya's Battlesong",
    "Moonstone Renewer", "Staff of Flowing Water", "Ardent Censer",
    "Chemtech Putrifier", "Chempunk Chainsword",
    "Serpent's Fang", "Edge of Night", "Youmuu's Ghostblade",
    "Umbral Glaive", "Manamune", "Seraph's Embrace",
];

/// Trinkets
const TRINKETS: &[&str] = &[
    "Stealth Ward", "Farsight Alteration", "Oracle Lens",
];

/// Sample summoner names
const SUMMONER_NAMES: &[&str] = &[
    "xXSlayerXx", "ProGamer123", "CloudNine", "ShadowStrike",
    "DragonSlayer", "NightHawk", "StormBringer", "PhoenixRising",
    "IronFist", "SwiftBlade", "DarkKnight", "LightBringer",
    "ThunderBolt", "FrostBite", "FlameWarden", "SteelHeart",
    "VoidWalker", "StarForge", "MoonShade", "SunBreaker",
    "WildCard", "SilentWind", "CrimsonBlade", "EmeraldDream",
    "GoldenEagle", "SilverFang", "BronzeShield", "DiamondEdge",
];

/// Game modes
const GAME_MODES: &[&str] = &[
    "CLASSIC", "ARAM", "URF",
];

/// Badges that can be earned
const BADGES: &[&str] = &[
    "MVP", "ACE", "First Blood", "Pentakill", "Quadrakill",
    "Triple Kill", "Double Kill", "Legendary", "Godlike",
    "Most Damage", "Most Gold", "Vision Score", "Comeback",
];

// ============================================================================
// TFT-specific constants
// ============================================================================

/// TFT champions/units (Set 12 "Magic n' Mayhem" themed names)
const TFT_UNITS: &[&str] = &[
    "Ahri", "Akali", "Blitzcrank", "Bard", "Briar", "Cassiopeia", "Diana",
    "Elise", "Ezreal", "Fiora", "Galio", "Gwen", "Hecarim", "Hwei", "Jax",
    "Jinx", "Karma", "Kassadin", "Katarina", "Kogmaw", "Lillia", "Morgana",
    "Neeko", "Nilah", "Nunu", "Olaf", "Poppy", "Rakan", "Rumble", "Ryze",
    "Seraphine", "Shen", "Shyvana", "Smolder", "Soraka", "Syndra", "Tahm Kench",
    "Taric", "Tristana", "Twitch", "Varus", "Veigar", "Vex", "Warwick",
    "Wukong", "Xerath", "Ziggs", "Zilean", "Zoe",
];

/// TFT traits (synergies)
const TFT_TRAITS: &[&str] = &[
    "Arcana", "Chrono", "Dragon", "Druid", "Eldritch", "Faerie", "Frost",
    "Honeymancy", "Hunter", "Incantor", "Mage", "Multistriker", "Preserver",
    "Pyro", "Scholar", "Shapeshifter", "Sugarcraft", "Vanguard", "Warrior",
    "Witchcraft", "Blaster", "Bastion",
];

/// TFT items
const TFT_ITEMS: &[&str] = &[
    "Bloodthirster", "Blue Buff", "Bramble Vest", "Deathblade",
    "Dragon's Claw", "Edge of Night", "Gargoyle Stoneplate", "Giant Slayer",
    "Guardbreaker", "Guinsoo's Rageblade", "Hand of Justice", "Hextech Gunblade",
    "Infinity Edge", "Ionic Spark", "Jeweled Gauntlet", "Last Whisper",
    "Morellonomicon", "Nashor's Tooth", "Quicksilver", "Rabadon's Deathcap",
    "Rapid Firecannon", "Redemption", "Runaan's Hurricane", "Spear of Shojin",
    "Statikk Shiv", "Steadfast Heart", "Sunfire Cape", "Thief's Gloves",
    "Titan's Resolve", "Warmog's Armor",
];

/// TFT augments (sample names)
const TFT_AUGMENTS: &[&str] = &[
    "Jeweled Lotus", "Buried Treasures", "Caretaker's Favor", "Component Grab Bag",
    "Cybernetic Implants", "Featherweights", "First Aid Kit", "Gold Reserves",
    "Idealism", "Investment", "Latent Forge", "Lucky Gloves", "Metabolic Accelerator",
    "Pandora's Items", "Portable Forge", "Pumping Up", "Scoped Weapons",
    "Silver Spoon", "Spoils of War", "Starter Kit", "Tiny Titans",
    "Trade Sector", "Unified Resistance", "Wellness Trust", "What the Forge",
    "You Have My Bow", "You Have My Sword", "Ascension", "Binary Airdrop",
    "Blue Battery", "Branching Out", "Built Different", "Cluttered Mind",
    "Combat Training", "Electrocharge", "Extended Duel", "Final Ascension",
    "Healing Orbs", "Level Up!", "Living Forge", "March of Progress",
    "Meditation", "Recombobulator", "Rich Get Richer", "Stand United",
    "Teaming Up", "The Golden Egg", "Think Fast", "Transfusion",
];

/// Generate sample League match data
pub fn generate_league_sample() -> Value {
    let mut rng = thread_rng();

    // Generate player's match data
    let player_name = SUMMONER_NAMES.choose(&mut rng).unwrap().to_string();
    let player_champion = CHAMPIONS.choose(&mut rng).unwrap().to_string();
    let is_win = rng.gen_bool(0.5);
    let result = if is_win { "win" } else { "loss" };

    // Generate KDA
    let kills = rng.gen_range(0..20);
    let deaths = rng.gen_range(0..15);
    let assists = rng.gen_range(0..25);

    // Generate other stats
    let champion_level = rng.gen_range(10..18);
    let duration_secs = rng.gen_range(1200..2400); // 20-40 minutes
    let duration_mins = duration_secs as f64 / 60.0;
    let cs = rng.gen_range(100..350);
    let cs_per_min = (cs as f64 / duration_mins * 10.0).round() / 10.0;
    let vision_score = rng.gen_range(10..80);
    let kill_participation = rng.gen_range(30..80);
    let damage_dealt = rng.gen_range(10000..50000) as i64;

    // Generate spells and runes
    let mut available_spells: Vec<&str> = SPELLS.to_vec();
    available_spells.shuffle(&mut rng);
    let spell1 = available_spells[0].to_string();
    let spell2 = available_spells[1].to_string();
    let keystone = KEYSTONES.choose(&mut rng).unwrap().to_string();
    let secondary_tree = RUNE_TREES.choose(&mut rng).unwrap().to_string();

    // Generate items (5-6 items)
    let num_items = rng.gen_range(5..=6);
    let mut available_items: Vec<&str> = ITEMS.to_vec();
    available_items.shuffle(&mut rng);
    let items: Vec<String> = available_items[..num_items]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let trinket = TRINKETS.choose(&mut rng).map(|s| s.to_string());

    // Generate game mode
    let game_mode = GAME_MODES.choose(&mut rng).unwrap().to_string();

    // Generate LP change for ranked
    let lp_change: Option<i32> = if game_mode == "CLASSIC" && rng.gen_bool(0.6) {
        Some(if is_win {
            rng.gen_range(15..25)
        } else {
            -rng.gen_range(10..20)
        })
    } else {
        None
    };

    // Generate rank
    let rank: Option<String> = if lp_change.is_some() {
        let tiers = ["Iron", "Bronze", "Silver", "Gold", "Platinum", "Emerald", "Diamond", "Master"];
        let divisions = ["IV", "III", "II", "I"];
        let tier = tiers.choose(&mut rng).unwrap();
        let division = divisions.choose(&mut rng).unwrap();
        Some(format!("{} {}", tier, division))
    } else {
        None
    };

    // Generate participants (10 players total, including the player)
    let mut used_champions: Vec<String> = vec![player_champion.clone()];
    let mut used_names: Vec<String> = vec![player_name.clone()];
    let player_team = if rng.gen_bool(0.5) { "blue" } else { "red" };

    let mut participants = Vec::new();

    // Add player
    participants.push(json!({
        "summonerName": player_name,
        "champion": player_champion,
        "team": player_team,
    }));

    // Add 4 teammates
    for _ in 0..4 {
        let name = get_unique_name(&mut rng, &used_names);
        used_names.push(name.clone());
        let champ = get_unique_champion(&mut rng, &used_champions);
        used_champions.push(champ.clone());

        participants.push(json!({
            "summonerName": name,
            "champion": champ,
            "team": player_team,
        }));
    }

    // Add 5 enemies
    let enemy_team = if player_team == "blue" { "red" } else { "blue" };
    for _ in 0..5 {
        let name = get_unique_name(&mut rng, &used_names);
        used_names.push(name.clone());
        let champ = get_unique_champion(&mut rng, &used_champions);
        used_champions.push(champ.clone());

        participants.push(json!({
            "summonerName": name,
            "champion": champ,
            "team": enemy_team,
        }));
    }

    // Generate badges (0-3 badges)
    let num_badges = rng.gen_range(0..=3);
    let mut available_badges: Vec<&str> = BADGES.to_vec();
    available_badges.shuffle(&mut rng);
    let badges: Vec<String> = available_badges[..num_badges]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Generate timestamps
    let played_at = Utc::now() - Duration::hours(rng.gen_range(1..48));
    let created_at = played_at + Duration::seconds(duration_secs as i64);

    // Build the match in V2 format (with core and details)
    json!({
        "core": {
            "id": format!("sample-{}", uuid::Uuid::new_v4()),
            "packId": "550e8400-e29b-41d4-a716-446655440000",
            "subpack": SUBPACK_LEAGUE,
            "externalMatchId": format!("{}", rng.gen_range(1000000000i64..9999999999i64)),
            "playedAt": played_at.to_rfc3339(),
            "durationSecs": duration_secs,
            "result": result,
            "isInProgress": false,
            "summarySource": "api",
            "createdAt": created_at.to_rfc3339(),
        },
        "details": {
            "summonerName": player_name,
            "champion": player_champion,
            "championLevel": champion_level,
            "kills": kills,
            "deaths": deaths,
            "assists": assists,
            "cs": cs,
            "csPerMin": cs_per_min,
            "visionScore": vision_score,
            "killParticipation": kill_participation,
            "damageDealt": damage_dealt,
            "gameMode": game_mode,
            "lpChange": lp_change,
            "rank": rank,
            "summonerSpell1": spell1,
            "summonerSpell2": spell2,
            "keystoneRune": keystone,
            "secondaryTree": secondary_tree,
            "items": items,
            "trinket": trinket,
            "participants": participants,
            "badges": badges,
        }
    })
}

/// Generate sample TFT match data
pub fn generate_tft_sample() -> Value {
    let mut rng = thread_rng();

    let player_name = SUMMONER_NAMES.choose(&mut rng).unwrap().to_string();
    let placement: u8 = rng.gen_range(1..=8);
    let is_win = placement <= 4;
    let result = if is_win { "win" } else { "loss" };
    let duration_secs = rng.gen_range(1500..2400); // 25-40 minutes

    // Player level (higher placements tend to have higher levels)
    let level: u8 = match placement {
        1..=2 => rng.gen_range(8..=10),
        3..=4 => rng.gen_range(7..=9),
        5..=6 => rng.gen_range(6..=8),
        _ => rng.gen_range(5..=7),
    };

    // Players eliminated (based on placement - 1st place eliminates more)
    let players_eliminated: u8 = match placement {
        1 => rng.gen_range(2..=4),
        2 => rng.gen_range(1..=3),
        3..=4 => rng.gen_range(0..=2),
        _ => rng.gen_range(0..=1),
    };

    // Total damage to players (higher placements = more damage typically)
    let total_damage: u32 = match placement {
        1 => rng.gen_range(80..120),
        2 => rng.gen_range(60..100),
        3..=4 => rng.gen_range(40..80),
        _ => rng.gen_range(20..60),
    };

    // Generate LP change for ranked
    let lp_change: Option<i32> = if rng.gen_bool(0.6) {
        Some(match placement {
            1 => rng.gen_range(35..50),
            2 => rng.gen_range(25..35),
            3 => rng.gen_range(15..25),
            4 => rng.gen_range(5..15),
            5 => -rng.gen_range(0..10),
            6 => -rng.gen_range(5..15),
            7 => -rng.gen_range(10..20),
            8 => -rng.gen_range(15..30),
            _ => 0,
        })
    } else {
        None
    };

    // Generate rank
    let rank: Option<String> = if lp_change.is_some() {
        let tiers = ["Iron", "Bronze", "Silver", "Gold", "Platinum", "Emerald", "Diamond", "Master"];
        let divisions = ["IV", "III", "II", "I"];
        let tier = tiers.choose(&mut rng).unwrap();
        let division = divisions.choose(&mut rng).unwrap();
        Some(format!("{} {}", tier, division))
    } else {
        None
    };

    // Generate units (board composition) - 7-9 units based on level
    let num_units = (level as usize).min(9);
    let mut available_units: Vec<&str> = TFT_UNITS.to_vec();
    available_units.shuffle(&mut rng);
    let units: Vec<Value> = available_units[..num_units]
        .iter()
        .map(|unit| {
            // Star level: 1-star common, 2-star less common, 3-star rare
            let tier: u8 = {
                let roll: f64 = rng.gen();
                if roll < 0.4 { 1 }
                else if roll < 0.85 { 2 }
                else { 3 }
            };

            // Items: 0-3 items per unit
            let num_items = rng.gen_range(0..=3);
            let mut available_items: Vec<&str> = TFT_ITEMS.to_vec();
            available_items.shuffle(&mut rng);
            let item_names: Vec<String> = available_items[..num_items]
                .iter()
                .map(|s| s.to_string())
                .collect();

            json!({
                "character": unit.to_string(),
                "tier": tier,
                "itemNames": item_names,
            })
        })
        .collect();

    // Generate traits (active synergies) - 4-7 active traits
    let num_traits = rng.gen_range(4..=7);
    let mut available_traits: Vec<&str> = TFT_TRAITS.to_vec();
    available_traits.shuffle(&mut rng);
    let traits: Vec<Value> = available_traits[..num_traits]
        .iter()
        .map(|trait_name| {
            let num_units = rng.gen_range(2..=6);
            let tier_current = rng.gen_range(1..=4);
            let tier_total = rng.gen_range(tier_current..=5);
            let style = match tier_current {
                1 => "bronze",
                2 => "silver",
                3 => "gold",
                _ => "chromatic",
            };

            json!({
                "name": trait_name.to_string(),
                "numUnits": num_units,
                "style": style,
                "tierCurrent": tier_current,
                "tierTotal": tier_total,
            })
        })
        .collect();

    // Generate augments (3 augments per game)
    let mut available_augments: Vec<&str> = TFT_AUGMENTS.to_vec();
    available_augments.shuffle(&mut rng);
    let augments: Vec<Value> = available_augments[..3]
        .iter()
        .enumerate()
        .map(|(i, aug)| {
            // First augment typically silver, second gold, third prismatic
            let tier = match i {
                0 => "silver",
                1 => "gold",
                _ => "prismatic",
            };
            json!({
                "name": aug.to_string(),
                "tier": tier,
            })
        })
        .collect();

    // TFT badges
    let tft_badges = ["Top 4", "First Place", "High Roller", "Perfect Game", "Comeback King"];
    let num_badges = rng.gen_range(0..=2);
    let mut available_badges: Vec<&str> = tft_badges.to_vec();
    available_badges.shuffle(&mut rng);
    let badges: Vec<String> = available_badges[..num_badges]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Generate timestamps
    let played_at = Utc::now() - Duration::hours(rng.gen_range(1..48));
    let created_at = played_at + Duration::seconds(duration_secs as i64);

    // Build the TFT match in V2 format
    json!({
        "core": {
            "id": format!("sample-{}", uuid::Uuid::new_v4()),
            "packId": "550e8400-e29b-41d4-a716-446655440000",
            "subpack": SUBPACK_TFT,
            "externalMatchId": format!("{}", rng.gen_range(1000000000i64..9999999999i64)),
            "playedAt": played_at.to_rfc3339(),
            "durationSecs": duration_secs,
            "result": result,
            "isInProgress": false,
            "summarySource": "api",
            "createdAt": created_at.to_rfc3339(),
        },
        "details": {
            "summonerName": player_name,
            "placement": placement,
            "gameMode": {
                "modeGuid": "TFT",
                "modeKey": "TFT",
                "displayName": "Teamfight Tactics",
                "queueId": 1100,
                "queueName": "Ranked TFT",
                "isRanked": lp_change.is_some(),
            },
            "lpChange": lp_change,
            "rank": rank,
            "badges": badges,
            "level": level,
            "playersEliminated": players_eliminated,
            "totalDamageToPlayers": total_damage,
            "traits": traits,
            "units": units,
            "augments": augments,
        }
    })
}

/// Get a unique champion name that hasn't been used yet
fn get_unique_champion(rng: &mut ThreadRng, used: &[String]) -> String {
    loop {
        let champ = CHAMPIONS.choose(rng).unwrap().to_string();
        if !used.contains(&champ) {
            return champ;
        }
    }
}

/// Get a unique summoner name that hasn't been used yet
fn get_unique_name(rng: &mut ThreadRng, used: &[String]) -> String {
    loop {
        let base = SUMMONER_NAMES.choose(rng).unwrap();
        // Add a random suffix to make names unique
        let name = format!("{}{}", base, rng.gen_range(1..999));
        if !used.contains(&name) {
            return name;
        }
    }
}

/// Generate sample match data for the specified subpack
pub fn generate_sample(subpack: u8) -> Option<Value> {
    match subpack {
        SUBPACK_LEAGUE => Some(generate_league_sample()),
        SUBPACK_TFT => Some(generate_tft_sample()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_league_sample() {
        let sample = generate_league_sample();

        // Check core fields exist
        assert!(sample.get("core").is_some());
        assert!(sample.get("details").is_some());

        let core = &sample["core"];
        assert!(core.get("id").is_some());
        assert!(core.get("result").is_some());
        assert!(core.get("durationSecs").is_some());

        let details = &sample["details"];
        assert!(details.get("champion").is_some());
        assert!(details.get("kills").is_some());
        assert!(details.get("deaths").is_some());
        assert!(details.get("assists").is_some());
        assert!(details.get("participants").is_some());

        // Check participants count
        let participants = details["participants"].as_array().unwrap();
        assert_eq!(participants.len(), 10);
    }

    #[test]
    fn test_generate_tft_sample() {
        let sample = generate_tft_sample();

        // Check core fields exist
        assert!(sample.get("core").is_some());
        assert!(sample.get("details").is_some());

        let core = &sample["core"];
        assert_eq!(core["subpack"], SUBPACK_TFT);

        let details = &sample["details"];
        assert!(details.get("placement").is_some());
        let placement = details["placement"].as_u64().unwrap();
        assert!((1..=8).contains(&placement));

        // Check new TFT fields
        assert!(details.get("level").is_some());
        let level = details["level"].as_u64().unwrap();
        assert!((5..=10).contains(&level));

        assert!(details.get("traits").is_some());
        let traits = details["traits"].as_array().unwrap();
        assert!(!traits.is_empty());

        assert!(details.get("units").is_some());
        let units = details["units"].as_array().unwrap();
        assert!(!units.is_empty());

        assert!(details.get("augments").is_some());
        let augments = details["augments"].as_array().unwrap();
        assert_eq!(augments.len(), 3);

        assert!(details.get("playersEliminated").is_some());
        assert!(details.get("totalDamageToPlayers").is_some());
    }

    #[test]
    fn test_generate_sample_subpack() {
        assert!(generate_sample(SUBPACK_LEAGUE).is_some());
        assert!(generate_sample(SUBPACK_TFT).is_some());
        assert!(generate_sample(99).is_none());
    }
}
