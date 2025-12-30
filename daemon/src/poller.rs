use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info};

use super::{LiveClientApi, ParsedGameEvent};
use crate::Result;

pub struct GamePoller {
    api: LiveClientApi,
    poll_interval: Duration,
    last_event_id: Arc<RwLock<i32>>,
}

impl GamePoller {
    pub fn new(poll_interval_ms: u64) -> Result<Self> {
        Ok(Self {
            api: LiveClientApi::new()?,
            poll_interval: Duration::from_millis(poll_interval_ms),
            last_event_id: Arc::new(RwLock::new(-1)),
        })
    }

    pub async fn start_polling(
        &self,
        event_tx: broadcast::Sender<ParsedGameEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        info!("Starting game event polling");

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.poll_interval) => {
                    if let Err(e) = self.poll_events(&event_tx).await {
                        debug!("Polling error (game may not be active): {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Stopping game event polling");
                    break;
                }
            }
        }
    }

    async fn poll_events(&self, event_tx: &broadcast::Sender<ParsedGameEvent>) -> Result<()> {
        let events = self.api.get_events().await?;
        let active_player = self.api.get_active_player().await?;
        let player_name = &active_player.summoner_name;

        let mut last_id = self.last_event_id.write().await;

        for event in events.events {
            if event.event_id <= *last_id {
                continue;
            }

            *last_id = event.event_id;

            let is_player_involved = event.killer_name.as_ref() == Some(player_name)
                || event.victim_name.as_ref() == Some(player_name)
                || event.assisters.contains(player_name);

            let parsed = ParsedGameEvent {
                event_type: super::LeagueEventType::from(event.event_name.as_str()),
                event_time: event.event_time,
                killer_name: event.killer_name,
                victim_name: event.victim_name,
                assisters: event.assisters,
                is_player_involved,
            };

            let _ = event_tx.send(parsed);
        }

        Ok(())
    }

    pub async fn reset(&self) {
        let mut last_id = self.last_event_id.write().await;
        *last_id = -1;
    }
}
