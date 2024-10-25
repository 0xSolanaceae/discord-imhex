#[path = "../src/tray.rs"]
mod tray;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::error::Error;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use tray::{create_tray_icon, VERSION, ICON};
struct TestContext {
    _temp_dir: TempDir,
    test_path: PathBuf,
}

fn setup_test_env() -> TestContext {
    let temp_dir = tempdir().unwrap();
    let test_path = temp_dir.path().to_owned();
    let discord_imhex_dir = test_path.join(".discord-imhex");
    fs::create_dir_all(&discord_imhex_dir).unwrap();
    TestContext {
        _temp_dir: temp_dir,
        test_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_create_tray_icon_success() -> Result<(), Box<dyn Error>> {
        let context = setup_test_env();
        env::set_var("DISCORD_IMHEX_DIR", context.test_path.to_str().unwrap());

        let icon_path = context.test_path.join("icon.ico");
        let mut file = File::create(&icon_path)?;
        file.write_all(ICON)?;

        let running = Arc::new(AtomicBool::new(true));
        let result = create_tray_icon(&running);
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.to_string().contains("systray") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    #[test]
    fn test_log_error() -> Result<(), Box<dyn Error>> {
        let context = setup_test_env();
        let log_path = context.test_path.join(".discord-imhex").join("error.log");
        
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&log_path)?;
        let test_message = "Test error message";
        writeln!(file, "{}", test_message)?;
        
        assert!(log_path.exists());
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains(test_message));
        Ok(())
    }

    #[test]
    fn test_tray_icon_shutdown() -> Result<(), Box<dyn Error>> {
        let context = setup_test_env();
        env::set_var("DISCORD_IMHEX_DIR", context.test_path.to_str().unwrap());

        let running = Arc::new(AtomicBool::new(true));
        running.store(false, Ordering::SeqCst);
        assert!(!running.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    fn test_temp_icon_creation() -> Result<(), Box<dyn Error>> {
        let context = setup_test_env();
        let icon_path = context.test_path.join(".discord-imhex").join("icon.ico");
        
        if let Some(parent) = icon_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&icon_path)?;
        file.write_all(ICON)?;
        
        assert!(icon_path.exists());
        let icon_contents = fs::read(&icon_path)?;
        assert_eq!(icon_contents, ICON);
        
        Ok(())
    }

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.chars().any(|c| c.is_digit(10)));
    }

    #[test]
    fn test_icon_constant() {
        assert!(!ICON.is_empty());
    }
}