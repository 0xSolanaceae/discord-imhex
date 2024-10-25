use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

use systray::{Application, Error as SystrayError};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ICON: &[u8] = include_bytes!("data/icon.ico");

pub fn create_tray_icon(running: &Arc<AtomicBool>) -> Result<Arc<Mutex<Application>>, Box<dyn Error>> {
    let app = Arc::new(Mutex::new(Application::new()?));

    let temp_icon_path = create_temp_icon_file()?;
    {
        let app = app.lock().unwrap();
        app.set_icon_from_file(temp_icon_path.to_str().unwrap())?;
        app.set_tooltip(&format!("discord-imhex v{}", VERSION))?;
    }

    add_menu_items(&app, running)?;

    let app_clone = Arc::clone(&app);
    thread::spawn(move || {
        app_clone.lock().unwrap().wait_for_message().expect("Failed to wait for message");
    });

    Ok(app)
}

fn create_temp_icon_file() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let mut temp_icon_path = env::temp_dir();
    temp_icon_path.push("icon.ico");

    let mut file = File::create(&temp_icon_path)?;
    file.write_all(ICON)?;

    Ok(temp_icon_path)
}

fn add_menu_items(app: &Arc<Mutex<Application>>, running: &Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    let app = Arc::clone(app);

    app.lock().unwrap().add_menu_item(&format!("discord-imhex v{}", VERSION), |_| -> Result<(), SystrayError> {
        let _ = open::that("https://github.com/0xSolanaceae/discord-imhex");
        Ok(())
    })?;

    app.lock().unwrap().add_menu_separator()?;

    app.lock().unwrap().add_menu_item("View Logs", |_| -> Result<(), SystrayError> {
        if let Ok(folder_path) = get_logs_folder() {
            let _ = open::that(folder_path);
        }
        Ok(())
    })?;

    let running_clone = Arc::clone(running);
    app.lock().unwrap().add_menu_item("Exit", move |_| -> Result<(), SystrayError> {
        running_clone.store(false, Ordering::SeqCst);
        std::process::exit(0);
    })?;

    Ok(())
}

fn get_logs_folder() -> Result<String, env::VarError> {
    let user_profile = env::var("USERPROFILE")?;
    Ok(format!("{}\\.discord-imhex", user_profile))
}
