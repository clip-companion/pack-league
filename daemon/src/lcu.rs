use crate::{AppError, Result};
use crate::GameflowPhase;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct LcuConnection {
    pub port: u16,
    pub auth_token: String,
    pub protocol: String,
}

impl LcuConnection {
    /// Find the League of Legends install directory by looking at the running process.
    /// This works regardless of where League is installed.
    fn find_install_directory() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            Self::find_install_directory_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::find_install_directory_macos()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(AppError::Other("Unsupported platform".into()))
        }
    }

    #[cfg(target_os = "windows")]
    fn find_install_directory_windows() -> Result<PathBuf> {
        use std::os::windows::process::CommandExt;

        // Use WMIC to find the LeagueClientUx.exe process and get its command line
        // CREATE_NO_WINDOW (0x08000000) prevents a console window from appearing
        let output = Command::new("WMIC")
            .args(["PROCESS", "WHERE", "name='LeagueClientUx.exe'", "GET", "commandline"])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to run WMIC: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::extract_install_directory(&stdout)
    }

    #[cfg(target_os = "macos")]
    fn find_install_directory_macos() -> Result<PathBuf> {
        // Use ps to find the LeagueClientUx process
        let ps_output = Command::new("ps")
            .args(["x", "-o", "args"])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Other(format!("Failed to run ps: {}", e)))?;

        let grep_output = Command::new("grep")
            .arg("LeagueClientUx")
            .stdin(ps_output.stdout.ok_or_else(|| AppError::Other("No stdout from ps".into()))?)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to run grep: {}", e)))?;

        let stdout = String::from_utf8_lossy(&grep_output.stdout);
        Self::extract_install_directory(&stdout)
    }

    /// Extract the install directory from the process command line.
    /// Looks for --install-directory= argument.
    fn extract_install_directory(cmdline: &str) -> Result<PathBuf> {
        // Match --install-directory=<path> (path may be quoted or unquoted)
        let re = Regex::new(r#"--install-directory[=\s]+["']?([^"'\s]+)["']?"#)
            .map_err(|e| AppError::Other(format!("Regex error: {}", e)))?;

        if let Some(caps) = re.captures(cmdline) {
            if let Some(path) = caps.get(1) {
                let install_dir = PathBuf::from(path.as_str());
                debug!("Found League install directory: {:?}", install_dir);
                return Ok(install_dir);
            }
        }

        // Fall back to default paths if we can't find the process
        Self::fallback_install_directory()
    }

    /// Fallback to well-known install paths
    fn fallback_install_directory() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        let paths = [
            "/Applications/League of Legends.app/Contents/LoL",
        ];

        #[cfg(target_os = "windows")]
        let paths = [
            "C:\\Riot Games\\League of Legends",
            "D:\\Riot Games\\League of Legends",
            "C:\\Program Files\\Riot Games\\League of Legends",
            "C:\\Program Files (x86)\\Riot Games\\League of Legends",
        ];

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let paths: [&str; 0] = [];

        for path in paths {
            let p = PathBuf::from(path);
            let lockfile = p.join("lockfile");
            if lockfile.exists() {
                debug!("Found League at fallback path: {:?}", p);
                return Ok(p);
            }
        }

        Err(AppError::LeagueNotRunning)
    }

    /// Connect to the LCU by finding and parsing the lockfile.
    /// Works on both macOS and Windows, regardless of install location.
    pub fn from_lockfile() -> Result<Self> {
        let install_dir = Self::find_install_directory()?;
        let lockfile_path = install_dir.join("lockfile");

        if !lockfile_path.exists() {
            return Err(AppError::LeagueNotRunning);
        }

        let content = std::fs::read_to_string(&lockfile_path)?;
        Self::parse_lockfile_content(&content)
    }

    /// Parse lockfile content into connection info.
    /// Format: process:pid:port:password:protocol
    fn parse_lockfile_content(content: &str) -> Result<Self> {
        let parts: Vec<&str> = content.trim().split(':').collect();

        if parts.len() < 5 {
            return Err(AppError::Other(format!(
                "Invalid lockfile format: expected 5 parts, got {}",
                parts.len()
            )));
        }

        let port = parts[2].parse().map_err(|_| {
            AppError::Other(format!("Invalid port in lockfile: {}", parts[2]))
        })?;

        info!("LCU connection: port={}, protocol={}", port, parts[4]);

        Ok(Self {
            port,
            auth_token: parts[3].to_string(),
            protocol: parts[4].to_string(),
        })
    }

    pub fn base_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.protocol, self.port)
    }

    /// Create Basic auth header value for LCU API
    pub fn auth_header(&self) -> String {
        let credentials = format!("riot:{}", self.auth_token);
        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
    }
}

/// LCU API client for communicating with the League Client
pub struct LcuClient {
    connection: LcuConnection,
    client: Client,
}

impl LcuClient {
    /// Create a new LCU client from lockfile
    pub fn new() -> Result<Self> {
        let connection = LcuConnection::from_lockfile()?;

        // LCU uses self-signed certs, so we need to disable cert verification
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| AppError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { connection, client })
    }

    /// Create from an existing connection
    pub fn from_connection(connection: LcuConnection) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| AppError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { connection, client })
    }

    /// Get the current gameflow phase
    pub async fn get_gameflow_phase(&self) -> Result<GameflowPhase> {
        let url = format!(
            "{}/lol-gameflow/v1/gameflow-phase",
            self.connection.base_url()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.connection.auth_header())
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to get gameflow phase: {}", e);
                AppError::LeagueNotRunning
            })?;

        if !response.status().is_success() {
            debug!("Gameflow phase request failed with status: {}", response.status());
            return Ok(GameflowPhase::None);
        }

        // The API returns a JSON string like "InProgress"
        let phase_str: String = response.json().await.map_err(|e| {
            warn!("Failed to parse gameflow phase: {}", e);
            AppError::Other(format!("Failed to parse gameflow phase: {}", e))
        })?;

        Ok(GameflowPhase::from(phase_str.as_str()))
    }

    /// Get the current summoner info
    pub async fn get_current_summoner(&self) -> Result<Summoner> {
        let url = format!(
            "{}/lol-summoner/v1/current-summoner",
            self.connection.base_url()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.connection.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("Failed to get summoner: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "Summoner request failed: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Failed to parse summoner: {}", e)))
    }

    /// Check if League client is running and connected
    pub async fn is_connected(&self) -> bool {
        self.get_gameflow_phase().await.is_ok()
    }

    /// Get end of game stats from LCU
    pub async fn get_end_of_game_stats(&self) -> Result<EndOfGameStats> {
        let url = format!(
            "{}/lol-end-of-game/v1/eog-stats-block",
            self.connection.base_url()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.connection.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("Failed to get EOG stats: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "EOG stats request failed: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Failed to parse EOG stats: {}", e)))
    }

    /// Get current ranked stats for the summoner
    pub async fn get_ranked_stats(&self) -> Result<Vec<RankedEntry>> {
        let summoner = self.get_current_summoner().await?;
        let url = format!(
            "{}/lol-ranked/v1/ranked-stats/{}",
            self.connection.base_url(),
            summoner.account_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.connection.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("Failed to get ranked stats: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "Ranked stats request failed: {}",
                response.status()
            )));
        }

        let stats: RankedStats = response
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Failed to parse ranked stats: {}", e)))?;

        Ok(stats.queues)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    pub account_id: i64,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
    pub summoner_level: i32,
    pub profile_icon_id: i32,
}

impl Summoner {
    /// Get the player's display name (Riot ID format: GameName#TagLine)
    pub fn riot_id(&self) -> String {
        if !self.game_name.is_empty() {
            if !self.tag_line.is_empty() {
                format!("{}#{}", self.game_name, self.tag_line)
            } else {
                self.game_name.clone()
            }
        } else {
            self.display_name.clone()
        }
    }
}

/// End of game statistics from LCU
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndOfGameStats {
    pub game_id: i64,
    pub game_mode: String,
    pub game_length: i32,
    pub game_type: String,
    pub local_player: Option<LocalPlayerStats>,
    pub teams: Vec<TeamStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalPlayerStats {
    pub champion_name: String,
    pub summoner_name: String,
    pub stats: PlayerStats,
    pub spell1_id: i32,
    pub spell2_id: i32,
    pub team_id: i32,
    pub items: Vec<i32>,
    pub perk0: i32,
    pub perk_sub_style: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats {
    pub assists: i32,
    pub champions_killed: i32,
    pub num_deaths: i32,
    pub minions_killed: i32,
    pub neutral_minions_killed: i32,
    pub vision_score: i32,
    pub total_damage_dealt_to_champions: i64,
    pub gold_earned: i32,
    pub level: i32,
    #[serde(default)]
    pub win: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamStats {
    pub team_id: i32,
    pub is_winning_team: bool,
    pub players: Vec<TeamPlayerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayerStats {
    pub champion_name: String,
    pub summoner_name: String,
    pub stats: PlayerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedStats {
    pub queues: Vec<RankedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedEntry {
    pub queue_type: String,
    pub tier: String,
    pub division: String,
    pub league_points: i32,
}
