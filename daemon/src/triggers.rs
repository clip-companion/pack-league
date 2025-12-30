use super::{LeagueEventType, ParsedGameEvent};
use crate::TriggerSettings;

#[derive(Clone)]
pub struct TriggerEvaluator {
    pub(crate) settings: TriggerSettings,
}

impl TriggerEvaluator {
    pub fn new(settings: TriggerSettings) -> Self {
        Self { settings }
    }

    pub fn update_settings(&mut self, settings: TriggerSettings) {
        self.settings = settings;
    }

    pub fn should_trigger(&self, event: &ParsedGameEvent) -> bool {
        if !event.is_player_involved {
            return false;
        }

        match event.event_type {
            LeagueEventType::ChampionKill => {
                if event.killer_name.is_some() && self.settings.on_kill {
                    return true;
                }
                if event.victim_name.is_some() && self.settings.on_death {
                    return true;
                }
                if !event.assisters.is_empty() && self.settings.on_assist {
                    return true;
                }
                false
            }
            LeagueEventType::Multikill => self.settings.on_multikill,
            LeagueEventType::Ace => self.settings.on_ace,
            LeagueEventType::TurretKilled => self.settings.on_tower_kill,
            LeagueEventType::DragonKill => self.settings.on_dragon,
            LeagueEventType::BaronKill => self.settings.on_baron,
            _ => false,
        }
    }

    pub fn get_trigger_name(&self, event: &ParsedGameEvent) -> String {
        match event.event_type {
            LeagueEventType::ChampionKill => {
                if event.killer_name.is_some() {
                    "kill".to_string()
                } else if event.victim_name.is_some() {
                    "death".to_string()
                } else {
                    "assist".to_string()
                }
            }
            LeagueEventType::Multikill => "multikill".to_string(),
            LeagueEventType::Ace => "ace".to_string(),
            LeagueEventType::TurretKilled => "tower".to_string(),
            LeagueEventType::DragonKill => "dragon".to_string(),
            LeagueEventType::BaronKill => "baron".to_string(),
            _ => "event".to_string(),
        }
    }
}
