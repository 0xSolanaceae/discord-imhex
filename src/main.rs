//#![windows_subsystem = "windows"]

mod imhex;
mod tray;
mod utils;
mod updater;

use discord_rich_presence::{activity::{Activity, Timestamps}, DiscordIpc, DiscordIpcClient};
use log::error;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use chrono::Local;
use tokio::runtime::Runtime;

pub struct DiscordClient {
    client: DiscordIpcClient,
}

impl DiscordClient {
    pub fn new(client_id: &str) -> Result<Self, Box<dyn Error>> {
        let mut client = DiscordIpcClient::new(client_id)?;
        client.connect()?;
        Ok(Self { client })
    }

    pub fn set_activity(&mut self, state: String, details: String, timestamps: Timestamps) -> Result<(), Box<dyn Error>> {
        let activity = Activity::new().state(&state).details(&details).timestamps(timestamps);
        self.client.set_activity(activity)
    }

    pub fn clear_activity(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.clear_activity()
    }
}

pub fn create_timestamps(start_time: i64) -> Timestamps {
    Timestamps::new().start(start_time)
}

fn setup_logging() -> Result<(), Box<dyn Error>> {
    let home_dir = std::env::var("USERPROFILE")?;
    let log_dir = Path::new(&home_dir).join(".discord-imhex");

    if !log_dir.exists() {
        fs::create_dir(&log_dir)?;
    }

    let log_file_path = log_dir.join("error.log");
    File::create(&log_file_path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    let timestamp = Local::now();
    writeln!(file, "[{}] Log file successfully created in {:?}", timestamp.format("%Y-%m-%d %H:%M:%S"), log_dir)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging()?;
    let client_id = "1060827018196955177";
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let _tray_icon = tray::create_tray_icon(&running_clone).map_err(|e| {
        error!("Failed to create tray icon: {}", e);
        e
    })?;

    let rt = Runtime::new()?;
    rt.spawn(updater::start_updater());

    let mut start_time: Option<i64> = None;
    let mut imhex_was_running = false;

    while running.load(Ordering::SeqCst) {
        match DiscordClient::new(client_id) {
            Ok(mut client) => {
                while running.load(Ordering::SeqCst) {
                    if imhex::is_imhex_running() {
                        handle_imhex_running(&mut client, &mut start_time, &mut imhex_was_running)?;
                    } else {
                        handle_imhex_not_running(&mut client, &mut imhex_was_running, &mut start_time)?;
                    }
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                client.clear_activity().map_err(|e| {
                    error!("Failed to clear activity: {}", e);
                    e
                })?;
            }
            Err(e) => {
                error!("Failed to connect to Discord client: {}", e);
                thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }

    Ok(())
}

fn handle_imhex_running(client: &mut DiscordClient, start_time: &mut Option<i64>, imhex_was_running: &mut bool) -> Result<(), Box<dyn Error>> {
    let current_time = utils::get_current_timestamp();

    if !*imhex_was_running {
        *start_time = Some(current_time);
        *imhex_was_running = true;
    }

    if let Some(current_opened_file) = imhex::check_if_imhex_window_exists() {
        let timestamps = create_timestamps(start_time.unwrap());
        let selected_bytes = imhex::get_selected_bytes().unwrap_or_else(|| "None".to_string());
        let state = format!("Bytes: [{}]", selected_bytes);
        let details = if current_opened_file == "ImHex" {
            "Idle".to_string()
        } else {
            format!("Analyzing: [{}]", current_opened_file)
        };

        client.set_activity(state, details, timestamps).map_err(|e| {
            error!("Failed to set activity: {}", e);
            e
        })?;
    } else {
        client.set_activity("".to_string(), "Idle".to_string(), Timestamps::default()).map_err(|e| {
            error!("Failed to set activity: {}", e);
            e
        })?;
    }

    Ok(())
}

fn handle_imhex_not_running(client: &mut DiscordClient, imhex_was_running: &mut bool, start_time: &mut Option<i64>) -> Result<(), Box<dyn Error>> {
    if *imhex_was_running {
        *imhex_was_running = false;
        *start_time = None;
        client.clear_activity().map_err(|e| {
            error!("Failed to clear activity: {}", e);
            e
        })?;
    }

    Ok(())
}