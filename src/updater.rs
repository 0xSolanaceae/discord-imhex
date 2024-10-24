use reqwest::Error;
use serde::Deserialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;
use semver::Version;
use dirs::home_dir;
use std::env;
use chrono::Local;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    browser_download_url: String,
}

pub async fn check_for_updates() -> Result<(), Error> {
    let url = "https://api.github.com/repos/0xSolanaceae/discord-imhex/releases/latest";
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "discord-imhex")
        .send()
        .await?
        .json::<Release>()
        .await?;

    let latest_version = response.tag_name.trim_start_matches('v');
    let current_version = env!("CARGO_PKG_VERSION").trim_start_matches('v');

    let latest_version = Version::parse(latest_version).expect("Invalid latest version format");
    let current_version = Version::parse(current_version).expect("Invalid current version format");

    if latest_version > current_version {
        log_message(&format!("Update available: v{} -> v{}", current_version, latest_version));
        if !response.assets.is_empty() {
            download_and_run_update(&response.assets[0].browser_download_url).await?;
        } else {
            log_message("No assets found for the latest release.");
        }
    } else {
        log_message(&format!("You are using the latest version: v{}", current_version));
    }

    Ok(())
}

pub async fn download_and_run_update(url: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.bytes().await?;

    let new_exe_path = "updated.exe";
    fs::write(new_exe_path, &response).expect("Failed to write new executable");

    let current_exe_path = env::current_exe().expect("Failed to get current executable path");

    let backup_exe_path = current_exe_path.with_extension("bak");
    fs::rename(&current_exe_path, &backup_exe_path).expect("Failed to rename current executable");

    fs::rename(new_exe_path, &current_exe_path).expect("Failed to replace current executable");

    log_message("Update installed successfully. Restarting application...");
    Command::new(current_exe_path)
        .spawn()
        .expect("Failed to restart application");

    std::process::exit(0);
}

pub async fn start_updater() {
    let mut interval = interval(Duration::from_secs(60 * 60 * 4));
    loop {
        interval.tick().await;
        if let Err(e) = check_for_updates().await {
            log_message(&format!("Failed to check for updates: {}", e));
        }
    }
}

pub fn log_message(message: &str) {
    if let Some(home_dir) = home_dir() {
        let log_path = home_dir.join(".discord-imhex").join("error.log");
        if let Some(log_dir) = log_path.parent() {
            fs::create_dir_all(log_dir).expect("Unable to create log directory");
        }
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Unable to open log file");
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(log_file, "[{}] {}", timestamp, message).expect("Unable to write to log file");
    } else {
        eprintln!("Unable to determine home directory");
    }
}