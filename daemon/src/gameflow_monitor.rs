//! Gameflow Phase Monitor
//!
//! Monitors League client gameflow phase changes via WebSocket (preferred)
//! or REST API polling (fallback), and notifies listeners when phases change.
//!
//! # Event Flow
//!
//! ```text
//! League Client Opens
//!        │
//!        ▼
//! ┌─────────────────┐
//! │  Try WebSocket  │──────┐
//! └────────┬────────┘      │ (if fails)
//!          │               │
//!          ▼               ▼
//! ┌─────────────────┐  ┌─────────────────┐
//! │ WebSocket Mode  │  │  Polling Mode   │
//! │ (real-time)     │  │  (1s interval)  │
//! └────────┬────────┘  └────────┬────────┘
//!          │                    │
//!          └────────┬───────────┘
//!                   ▼
//!          Phase Change Detected
//!                   │
//!                   ▼
//!          Emit Tauri Events:
//!          - "gameflow-change"
//!          - "stage-change"
//! ```

use anyhow::Result;
use crate::{GameflowPhase, LcuClient, LcuWebSocket, LcuEvent, uris};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

/// The target layout based on gameflow phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetLayout {
    /// No layout - League not active
    None,
    /// Client window centered on stage
    ClientCentered,
    /// Game fills entire stage
    GameFullscreen,
}

impl TargetLayout {
    /// Get the layout name as used by StageManager
    pub fn layout_name(&self) -> Option<&'static str> {
        match self {
            TargetLayout::None => None,
            TargetLayout::ClientCentered => Some("client_centered"),
            TargetLayout::GameFullscreen => Some("game_fullscreen"),
        }
    }

    /// Determine target layout from gameflow phase
    pub fn from_phase(phase: GameflowPhase) -> Self {
        if phase.is_in_game() {
            TargetLayout::GameFullscreen
        } else if phase.is_in_client() {
            TargetLayout::ClientCentered
        } else {
            TargetLayout::None
        }
    }
}

/// Event emitted when the stage layout should change
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageChangeEvent {
    /// The new target layout
    pub layout: String,
    /// The gameflow phase that triggered the change
    pub phase: String,
    /// Human-readable reason for the change
    pub reason: String,
}

/// Event emitted for gameflow phase changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameflowChangeEvent {
    /// The new gameflow phase
    pub phase: String,
    /// Display name for the phase
    pub display_name: String,
    /// Whether we're in game
    pub is_in_game: bool,
    /// Whether we're in the client
    pub is_in_client: bool,
}

/// Events emitted by the gameflow monitor
#[derive(Debug, Clone)]
pub enum GameflowEvent {
    /// Gameflow phase changed
    PhaseChanged(GameflowChangeEvent),
    /// Stage layout should change
    StageChanged(StageChangeEvent),
}

/// Monitor mode - WebSocket (preferred) or Polling (fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MonitorMode {
    /// Real-time events via WebSocket
    WebSocket,
    /// Polling the REST API
    Polling,
}

/// Monitor for League client gameflow phase changes
pub struct GameflowMonitor {
    poll_interval: Duration,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl GameflowMonitor {
    /// Create a new gameflow monitor
    pub fn new(poll_interval_ms: u64) -> Self {
        Self {
            poll_interval: Duration::from_millis(poll_interval_ms),
            shutdown_tx: None,
        }
    }

    /// Create with default 1-second polling interval (only used as fallback)
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Start monitoring gameflow changes
    ///
    /// Prefers WebSocket for real-time events, falls back to polling if unavailable.
    ///
    /// Sends events via the provided channel when:
    /// - The gameflow phase changes
    /// - The stage layout should change
    pub async fn start(&mut self, event_tx: mpsc::Sender<GameflowEvent>) -> Result<()> {
        if self.shutdown_tx.is_some() {
            warn!("Gameflow monitor already running");
            return Ok(());
        }

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let poll_interval = self.poll_interval;

        tokio::spawn(async move {
            run_monitor_loop(event_tx, poll_interval, shutdown_rx).await;
        });

        info!("Gameflow monitor started (WebSocket preferred, {}ms polling fallback)",
              self.poll_interval.as_millis());
        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            info!("Gameflow monitor stopped");
        }
    }

    /// Check if the monitor is running
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

impl Drop for GameflowMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Main monitoring loop - tries WebSocket first, falls back to polling
async fn run_monitor_loop(
    event_tx: mpsc::Sender<GameflowEvent>,
    poll_interval: Duration,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut last_phase = GameflowPhase::None;
    let mut last_layout = TargetLayout::None;
    let mut mode = MonitorMode::Polling;
    let mut reconnect_delay = Duration::from_secs(1);

    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv().is_ok() {
            info!("Gameflow monitor shutdown signal received");
            break;
        }

        // Try to use WebSocket mode
        match try_websocket_mode(&event_tx, &mut last_phase, &mut last_layout, &mut shutdown_rx).await {
            Ok(()) => {
                // WebSocket closed gracefully, try to reconnect
                info!("WebSocket disconnected, will reconnect...");
                mode = MonitorMode::WebSocket;
                reconnect_delay = Duration::from_secs(1);
            }
            Err(e) => {
                // WebSocket failed to connect, use polling
                if mode == MonitorMode::WebSocket {
                    warn!("WebSocket unavailable: {}, falling back to polling", e);
                    mode = MonitorMode::Polling;
                }
            }
        }

        // Fall back to polling mode
        let poll_result = try_polling_mode(
            &event_tx,
            &mut last_phase,
            &mut last_layout,
            poll_interval,
            &mut shutdown_rx,
        ).await;

        match poll_result {
            PollResult::Shutdown => break,
            PollResult::ClientConnected => {
                // Client is running, try WebSocket again
                info!("League client detected, attempting WebSocket connection...");
                reconnect_delay = Duration::from_secs(1);
            }
            PollResult::ClientDisconnected => {
                // Client not running, wait before checking again
                tokio::time::sleep(reconnect_delay).await;
                reconnect_delay = (reconnect_delay * 2).min(Duration::from_secs(30));
            }
        }
    }
}

/// Result of polling attempt
enum PollResult {
    Shutdown,
    ClientConnected,
    ClientDisconnected,
}

/// Try to monitor via WebSocket (real-time events)
async fn try_websocket_mode(
    event_tx: &mpsc::Sender<GameflowEvent>,
    last_phase: &mut GameflowPhase,
    last_layout: &mut TargetLayout,
    shutdown_rx: &mut broadcast::Receiver<()>,
) -> Result<()> {
    let mut ws = LcuWebSocket::connect().await?;
    info!("Gameflow monitor using WebSocket mode (real-time events)");

    loop {
        tokio::select! {
            event = ws.recv() => {
                match event {
                    Some(event) => {
                        if let Some(phase) = parse_gameflow_event(&event) {
                            handle_phase_change(event_tx, phase, last_phase, last_layout).await;
                        }
                    }
                    None => {
                        // WebSocket closed
                        return Ok(());
                    }
                }
            }

            _ = shutdown_rx.recv() => {
                return Ok(());
            }
        }
    }
}

/// Try to monitor via REST API polling (fallback)
async fn try_polling_mode(
    event_tx: &mpsc::Sender<GameflowEvent>,
    last_phase: &mut GameflowPhase,
    last_layout: &mut TargetLayout,
    poll_interval: Duration,
    shutdown_rx: &mut broadcast::Receiver<()>,
) -> PollResult {
    let mut consecutive_failures = 0;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {
                match LcuClient::new() {
                    Ok(client) => {
                        match client.get_gameflow_phase().await {
                            Ok(phase) => {
                                if consecutive_failures > 0 {
                                    info!("Connected to League client (polling mode)");
                                }
                                consecutive_failures = 0;
                                handle_phase_change(event_tx, phase, last_phase, last_layout).await;

                                // Client is connected - try upgrading to WebSocket
                                return PollResult::ClientConnected;
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                if consecutive_failures == 1 {
                                    debug!("Failed to get gameflow phase: {}", e);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        consecutive_failures += 1;
                        if consecutive_failures == 1 {
                            debug!("League client not running");
                        }

                        // If we had a layout before, emit that it's now none
                        if *last_layout != TargetLayout::None {
                            handle_phase_change(event_tx, GameflowPhase::None, last_phase, last_layout).await;
                        }

                        // After several failures, wait longer
                        if consecutive_failures > 5 {
                            return PollResult::ClientDisconnected;
                        }
                    }
                }
            }

            _ = shutdown_rx.recv() => {
                return PollResult::Shutdown;
            }
        }
    }
}

/// Parse a gameflow phase from an LCU WebSocket event
fn parse_gameflow_event(event: &LcuEvent) -> Option<GameflowPhase> {
    if event.uri == uris::GAMEFLOW_PHASE {
        // The data is a JSON string like "InProgress"
        let phase_str = event.data.as_str()?;
        Some(GameflowPhase::from(phase_str))
    } else {
        None
    }
}

/// Handle a phase change - send events if the phase or layout changed
async fn handle_phase_change(
    event_tx: &mpsc::Sender<GameflowEvent>,
    phase: GameflowPhase,
    last_phase: &mut GameflowPhase,
    last_layout: &mut TargetLayout,
) {
    if phase != *last_phase {
        info!("Gameflow phase changed: {:?} -> {:?}", last_phase, phase);

        // Send gameflow change event
        let _ = event_tx.send(GameflowEvent::PhaseChanged(GameflowChangeEvent {
            phase: format!("{:?}", phase),
            display_name: phase.display_name().to_string(),
            is_in_game: phase.is_in_game(),
            is_in_client: phase.is_in_client(),
        })).await;

        // Check if layout should change
        let new_layout = TargetLayout::from_phase(phase);
        if new_layout != *last_layout {
            send_stage_change(event_tx, new_layout, phase).await;
            *last_layout = new_layout;
        }

        *last_phase = phase;
    }
}

/// Send a stage change event
async fn send_stage_change(event_tx: &mpsc::Sender<GameflowEvent>, layout: TargetLayout, phase: GameflowPhase) {
    let layout_name = layout.layout_name().unwrap_or("none").to_string();
    let reason = match layout {
        TargetLayout::None => "League client not active".to_string(),
        TargetLayout::ClientCentered => format!("Client phase: {}", phase.display_name()),
        TargetLayout::GameFullscreen => format!("In game: {}", phase.display_name()),
    };

    info!("Stage layout change: {} ({})", layout_name, reason);

    let _ = event_tx.send(GameflowEvent::StageChanged(StageChangeEvent {
        layout: layout_name,
        phase: format!("{:?}", phase),
        reason,
    })).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_layout_from_phase() {
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::InProgress),
            TargetLayout::GameFullscreen
        );
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::Reconnect),
            TargetLayout::GameFullscreen
        );
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::GameStart),
            TargetLayout::GameFullscreen
        );
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::ChampSelect),
            TargetLayout::ClientCentered
        );
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::Lobby),
            TargetLayout::ClientCentered
        );
        assert_eq!(
            TargetLayout::from_phase(GameflowPhase::None),
            TargetLayout::None
        );
    }

    #[test]
    fn test_layout_names() {
        assert_eq!(TargetLayout::ClientCentered.layout_name(), Some("client_centered"));
        assert_eq!(TargetLayout::GameFullscreen.layout_name(), Some("game_fullscreen"));
        assert_eq!(TargetLayout::None.layout_name(), None);
    }
}
