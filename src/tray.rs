use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

use systray::{Application, Error as SystrayError};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ICON: &[u8] = include_bytes!("data/icon.ico");

pub fn create_tray_icon(running: &Arc<AtomicBool>) -> Result<Arc<Mutex<Application>>, Box<dyn Error>> {
    let app = initialize_application()?;
    set_icon_and_tooltip(&app)?;

    add_menu_items(&app, running)?;
    spawn_message_listener(app.clone());

    Ok(app)
}

fn initialize_application() -> Result<Arc<Mutex<Application>>, SystrayError> {
    Ok(Arc::new(Mutex::new(Application::new()?)))
}

fn set_icon_and_tooltip(app: &Arc<Mutex<Application>>) -> Result<(), Box<dyn Error>> {
    let temp_icon_path = create_temp_icon_file()?;
    let app = app.lock().unwrap();
    app.set_icon_from_file(temp_icon_path.to_str().unwrap())?;
    app.set_tooltip(&format!("discord-imhex v{}", VERSION))?;
    Ok(())
}

fn create_temp_icon_file() -> Result<PathBuf, Box<dyn Error>> {
    let mut temp_icon_path = env::temp_dir();
    temp_icon_path.push("icon.ico");

    let mut file = File::create(&temp_icon_path)?;
    file.write_all(ICON)?;

    Ok(temp_icon_path)
}

fn add_menu_items(app: &Arc<Mutex<Application>>, running: &Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    let mut app = app.lock().unwrap();
    app.add_menu_item(&format!("discord-imhex v{}", VERSION), |_| {
        open::that("https://github.com/0xSolanaceae/discord-imhex").map_err(|_| SystrayError::OsError("Failed to open URL".into()))
    })?;

    app.add_menu_separator()?;
    app.add_menu_item("View Logs", |_| {
        if let Ok(folder_path) = get_logs_folder() {
            open::that(folder_path).map_err(|_| SystrayError::OsError("Failed to open logs folder".into()))
        } else {
            Ok(())
        }
    })?;

    let running_clone = Arc::clone(running);
    app.add_menu_item("Exit", move |_| -> Result<(), SystrayError> {
        running_clone.store(false, Ordering::SeqCst);
        std::process::exit(0);
    })?;

    Ok(())
}

fn spawn_message_listener(app: Arc<Mutex<Application>>) {
    thread::spawn(move || {
        app.lock().unwrap().wait_for_message().expect("Failed to wait for message");
    });
}

fn get_logs_folder() -> Result<String, env::VarError> {
    env::var("USERPROFILE").map(|user_profile| format!("{}\\.discord-imhex", user_profile))
}