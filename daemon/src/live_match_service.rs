use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::Result;
use crate::LiveMatch;

use super::LiveClientApi;

/// Events emitted by the live match service
#[derive(Debug, Clone)]
pub enum LiveMatchEvent {
    /// Live match data update
    Update(LiveMatch),
    /// Match ended (clears the live match)
    Ended,
}

/// Service that streams live match data during active games
pub struct LiveMatchService {
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl LiveMatchService {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
        }
    }

    /// Start streaming live match data
    pub async fn start(&mut self, event_tx: mpsc::Sender<LiveMatchEvent>) -> Result<()> {
        if self.shutdown_tx.is_some() {
            warn!("LiveMatchService already running");
            return Ok(());
        }

        info!("Starting LiveMatchService");

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        tokio::spawn(async move {
            let api = match LiveClientApi::new() {
                Ok(api) => api,
                Err(e) => {
                    error!("Failed to create LiveClientApi: {}", e);
                    return;
                }
            };

            let mut poll_interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("LiveMatchService shutting down");
                        break;
                    }
                    _ = poll_interval.tick() => {
                        match Self::poll_and_emit(&api, &event_tx).await {
                            Ok(()) => {}
                            Err(e) => {
                                debug!("Failed to poll live match data: {}", e);
                                // Don't break on error - game might still be loading
                            }
                        }
                    }
                }
            }

            // Send ended event to clear the live match
            let _ = event_tx.send(LiveMatchEvent::Ended).await;
        });

        Ok(())
    }

    /// Stop streaming live match data
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
            info!("LiveMatchService stopped");
        }
        Ok(())
    }

    /// Poll the Live Client API and send an update event
    async fn poll_and_emit(api: &LiveClientApi, event_tx: &mpsc::Sender<LiveMatchEvent>) -> Result<()> {
        let game_data = match api.get_all_game_data().await {
            Ok(data) => data,
            Err(e) => {
                warn!("Failed to get game data: {}", e);
                return Err(e);
            }
        };

        match LiveMatch::from_game_data(&game_data) {
            Some(live_match) => {
                if let Err(e) = event_tx.send(LiveMatchEvent::Update(live_match.clone())).await {
                    warn!("Failed to send live-match-update event: {}", e);
                }
                info!(
                    "Live update: {} on {} ({:.0}s) - CS={}, {} items, spells={:?}/{:?}, rune={:?}",
                    live_match.summoner_name,
                    live_match.champion,
                    live_match.game_time_secs,
                    live_match.cs,
                    live_match.items.len(),
                    live_match.spell1.as_ref().map(|s| &s.name),
                    live_match.spell2.as_ref().map(|s| &s.name),
                    live_match.runes.as_ref().map(|r| &r.keystone_name)
                );
            }
            None => {
                warn!("Failed to create LiveMatch from game data - active player not found?");
            }
        }

        Ok(())
    }

    /// Check if the service is currently running
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

impl Default for LiveMatchService {
    fn default() -> Self {
        Self::new()
    }
}
