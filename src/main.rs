#![windows_subsystem = "windows"]

pub mod imhex;
pub mod tray;
pub mod utils;
pub mod updater;

use discord_rich_presence::{activity::{Activity, Timestamps}, DiscordIpc, DiscordIpcClient};
use log::{error, info};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio::runtime::Runtime;

const CLIENT_ID: &str = "1060827018196955177";
const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum AppError {
    Discord(String),
    Filesystem(std::io::Error),
    Configuration(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Discord(msg) => write!(f, "Discord error: {}", msg),
            AppError::Filesystem(err) => write!(f, "Filesystem error: {}", err),
            AppError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Filesystem(err)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ActivityState {
    state: String,
    details: String,
}

struct Config {
    client_id: String,
    log_dir: PathBuf,
    update_interval: Duration,
}

impl Config {
    fn new() -> Result<Self, AppError> {
        let home_dir = std::env::var("USERPROFILE")
            .map_err(|_| AppError::Configuration("Failed to get user profile".to_string()))?;
        
        Ok(Config {
            client_id: CLIENT_ID.to_string(),
            log_dir: PathBuf::from(home_dir).join(".discord-imhex"),
            update_interval: UPDATE_INTERVAL,
        })
    }
}

pub struct DiscordClient {
    client: DiscordIpcClient,
    last_activity: Option<ActivityState>,
}

impl DiscordClient {
    pub fn new(client_id: &str) -> Result<Self, AppError> {
        let mut client = DiscordIpcClient::new(client_id)
            .map_err(|e| AppError::Discord(e.to_string()))?;
        client.connect()
            .map_err(|e| AppError::Discord(e.to_string()))?;
        
        Ok(Self { 
            client,
            last_activity: None,
        })
    }

    pub fn update_activity(&mut self, state: String, details: String, timestamps: Timestamps) -> Result<(), AppError> {
        let new_activity = ActivityState {
            state: state.clone(),
            details: details.clone(),
        };

        if Some(&new_activity) != self.last_activity.as_ref() {
            let activity = Activity::new()
                .state(&state)
                .details(&details)
                .timestamps(timestamps);

            self.client.set_activity(activity)
                .map_err(|e| AppError::Discord(e.to_string()))?;
            self.last_activity = Some(new_activity);
        }
        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<(), AppError> {
        self.client.clear_activity()
            .map_err(|e| AppError::Discord(e.to_string()))?;
        self.last_activity = None;
        Ok(())
    }
}

struct AppState {
    running: Arc<AtomicBool>,
    start_time: Option<i64>,
    imhex_running: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            start_time: None,
            imhex_running: false,
        }
    }
}

fn setup_logging(log_dir: &Path) -> Result<(), AppError> {
    if !log_dir.exists() {
        fs::create_dir(log_dir)?;
    }

    let log_file_path = log_dir.join("error.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    let timestamp = Local::now();
    writeln!(file, "[{}] Log file successfully created in {:?}", 
             timestamp.format("%Y-%m-%d %H:%M:%S"), log_dir)?;

    Ok(())
}

fn create_timestamps(start_time: i64) -> Timestamps {
    Timestamps::new().start(start_time)
}

fn handle_imhex_running(client: &mut DiscordClient, state: &mut AppState) -> Result<(), AppError> {
    let current_time = utils::get_current_timestamp();

    if !state.imhex_running {
        state.start_time = Some(current_time);
        state.imhex_running = true;
    }

    if let Some(current_opened_file) = imhex::check_if_imhex_window_exists() {
        let timestamps = create_timestamps(state.start_time.unwrap());
        let selected_bytes = imhex::get_selected_bytes().unwrap_or_else(|| "None".to_string());
        let activity_state = format!("Bytes: [{}]", selected_bytes);
        let details = if current_opened_file == "ImHex" {
            "Idle".to_string()
        } else {
            format!("Analyzing: [{}]", current_opened_file)
        };

        client.update_activity(activity_state, details, timestamps)?;
    } else {
        client.update_activity("".to_string(), "Idle".to_string(), Timestamps::default())?;
    }

    Ok(())
}

fn handle_imhex_not_running(client: &mut DiscordClient, state: &mut AppState) -> Result<(), AppError> {
    if state.imhex_running {
        state.imhex_running = false;
        state.start_time = None;
        client.clear_activity()?;
    }
    Ok(())
}

fn run_discord_loop(client: &mut DiscordClient, state: &mut AppState, config: &Config) -> Result<(), AppError> {
    while state.running.load(Ordering::SeqCst) {
        if imhex::is_imhex_running() {
            handle_imhex_running(client, state)?;
        } else {
            handle_imhex_not_running(client, state)?;
        }
        thread::sleep(config.update_interval);
    }
    client.clear_activity()?;
    Ok(())
}

fn main() -> Result<(), AppError> {
    let config = Config::new()?;
    setup_logging(&config.log_dir)?;
    
    let mut state = AppState::new();
    let running_clone = Arc::clone(&state.running);

    let _tray_icon = tray::create_tray_icon(&running_clone)
        .map_err(|e| AppError::Configuration(e.to_string()))?;

    let rt = Runtime::new()
        .map_err(|e| AppError::Configuration(e.to_string()))?;
    rt.spawn(updater::start_updater());

    info!("Application started successfully");

    while state.running.load(Ordering::SeqCst) {
        match DiscordClient::new(&config.client_id) {
            Ok(mut client) => {
                if let Err(e) = run_discord_loop(&mut client, &mut state, &config) {
                    error!("Error in Discord loop: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to connect to Discord client: {}", e);
                thread::sleep(RECONNECT_DELAY);
            }
        }
    }

    info!("Application shutting down");
    Ok(())
}