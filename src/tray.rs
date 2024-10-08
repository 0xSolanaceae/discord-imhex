use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::thread;

use chrono::Local;
use systray::{Application, Error as SystrayError};

//FIXME add update search for releases tab on github
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ICON: &[u8] = include_bytes!("data/icon.ico");

fn log_error(message: &str) {
    let log_file_path = ".discord-imhex/error.log";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}

pub fn create_tray_icon(running: &Arc<AtomicBool>) -> Result<Arc<Mutex<Application>>, Box<dyn Error>> {
    let running = Arc::clone(running);
    let app = Arc::new(Mutex::new(Application::new().map_err(|e| {
        log_error(&format!("Failed to create application: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
        e
    })?));

    let mut temp_icon_path = env::temp_dir();
    temp_icon_path.push("icon.ico");
    {
        let mut temp_icon_file = File::create(&temp_icon_path).map_err(|e| {
            log_error(&format!("Failed to create temp icon file: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
        temp_icon_file.write_all(ICON).map_err(|e| {
            log_error(&format!("Failed to write to temp icon file: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
    }

    {
        let app = app.lock().unwrap();
        app.set_icon_from_file(temp_icon_path.to_str().unwrap()).map_err(|e| {
            log_error(&format!("Failed to set icon from file: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
        app.set_tooltip(&format!("discord-imhex v{}", VERSION)).map_err(|e| {
            log_error(&format!("Failed to set tooltip: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
    }

    {
        let app = Arc::clone(&app);
        app.lock().unwrap().add_menu_item(&format!("discord-imhex v{}", VERSION), |_| -> Result<(), SystrayError> {
            let _ = open::that("https://github.com/0xSolanaceae/discord-imhex");
            Ok(())
        }).map_err(|e| {
            log_error(&format!("Failed to add menu item: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
        app.lock().unwrap().add_menu_separator().map_err(|e| {
            log_error(&format!("Failed to add menu separator: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
    }

    {
        let app = Arc::clone(&app);
        app.lock().unwrap().add_menu_item("View Logs", |_| -> Result<(), SystrayError> {
            let user_profile = env::var("USERPROFILE").unwrap();
            let folder_path = format!("{}\\.discord-imhex", user_profile);
            let _ = open::that(folder_path);
            Ok(())
        }).map_err(|e| {
            log_error(&format!("Failed to add open folder menu item: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
    }

    {
        let app = Arc::clone(&app);
        app.lock().unwrap().add_menu_item("Exit", move |_| -> Result<(), SystrayError> {
            running.store(false, Ordering::SeqCst);
            std::process::exit(0);
        }).map_err(|e| {
            log_error(&format!("Failed to add exit menu item: {} at {}", e, Local::now().format("%Y-%m-%d %H:%M:%S")));
            e
        })?;
    }

    let app_clone = Arc::clone(&app);
    thread::spawn(move || {
        app_clone.lock().unwrap().wait_for_message().expect("Failed to wait for message");
    });

    Ok(app)
}