//! League Pack Daemon Entry Point
//!
//! Standalone binary that communicates with the main daemon via NDJSON over stdin/stdout.
//! This is spawned as a subprocess by the main daemon's PackManager.

use std::io::{self, BufRead, Write};

use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

mod protocol;

use protocol::{GamepackCommand, GamepackResponse};

/// Protocol version - increment when breaking changes are made
const PROTOCOL_VERSION: u32 = 1;

/// Game ID for League of Legends
const GAME_ID: i32 = 1;

/// Game slug
const SLUG: &str = "league";

fn main() {
    // Initialize logging to stderr (stdout is reserved for protocol)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("pack_league=debug".parse().unwrap()))
        .with_writer(io::stderr)
        .init();

    info!("League pack daemon starting (protocol v{})", PROTOCOL_VERSION);

    // Run the main loop
    if let Err(e) = run_ipc_loop() {
        error!("IPC loop error: {}", e);
        std::process::exit(1);
    }

    info!("League pack daemon shutting down");
}

fn run_ipc_loop() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // TODO: Initialize the League integration
    // let runtime = tokio::runtime::Runtime::new()?;
    // let mut integration = runtime.block_on(async { LeagueIntegration::new() });

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                warn!("Failed to read stdin: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        debug!("Received command: {}", line);

        let cmd: GamepackCommand = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let response = GamepackResponse::Error {
                    request_id: "unknown".to_string(),
                    message: format!("Failed to parse command: {}", e),
                    code: Some("PARSE_ERROR".to_string()),
                };
                send_response(&mut stdout, &response);
                continue;
            }
        };

        let response = handle_command(cmd);
        send_response(&mut stdout, &response);

        // Check for shutdown
        if matches!(response, GamepackResponse::ShutdownComplete { .. }) {
            break;
        }
    }

    Ok(())
}

fn handle_command(cmd: GamepackCommand) -> GamepackResponse {
    match cmd {
        GamepackCommand::Init { request_id } => {
            info!("Initializing League integration");
            GamepackResponse::Initialized {
                request_id,
                game_id: GAME_ID,
                slug: SLUG.to_string(),
                protocol_version: PROTOCOL_VERSION,
            }
        }

        GamepackCommand::DetectRunning { request_id } => {
            // TODO: Actually detect if League client is running
            GamepackResponse::RunningStatus {
                request_id,
                running: false,
            }
        }

        GamepackCommand::GetStatus { request_id } => {
            // TODO: Get actual status from integration
            GamepackResponse::GameStatus {
                request_id,
                connected: false,
                connection_status: "disconnected".to_string(),
                game_phase: None,
                is_in_game: false,
            }
        }

        GamepackCommand::PollEvents { request_id } => {
            // TODO: Poll actual events from integration
            GamepackResponse::Events {
                request_id,
                events: vec![],
            }
        }

        GamepackCommand::GetLiveData { request_id } => {
            // TODO: Get actual live data from integration
            GamepackResponse::LiveData {
                request_id,
                data: None,
            }
        }

        GamepackCommand::SessionStart { request_id } => {
            // TODO: Start session with integration
            info!("Session starting");
            GamepackResponse::SessionStarted {
                request_id,
                context: None,
            }
        }

        GamepackCommand::SessionEnd { request_id, context: _ } => {
            // TODO: End session with integration
            info!("Session ending");
            GamepackResponse::SessionEnded {
                request_id,
                match_data: None,
            }
        }

        GamepackCommand::Shutdown { request_id } => {
            info!("Shutdown requested");
            GamepackResponse::ShutdownComplete { request_id }
        }
    }
}

fn send_response(stdout: &mut io::Stdout, response: &GamepackResponse) {
    if let Ok(json) = serde_json::to_string(response) {
        debug!("Sending response: {}", json);
        if let Err(e) = writeln!(stdout, "{}", json) {
            error!("Failed to write response: {}", e);
        }
        if let Err(e) = stdout.flush() {
            error!("Failed to flush stdout: {}", e);
        }
    }
}
