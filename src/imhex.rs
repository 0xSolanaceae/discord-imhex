use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;
use std::path::PathBuf;

use crate::utils::current_timestamp;
use lazy_static::lazy_static;
use chrono::Local;
use dirs::home_dir;

use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::HWND;
use winapi::um::winbase::CREATE_NO_WINDOW;
use winapi::um::winuser::{EnumWindows, GetWindowTextW};

lazy_static! {
    static ref PREVIOUS_TITLE: Mutex<Option<String>> = Mutex::new(None);
    static ref PREVIOUS_RUNNING_STATE: Mutex<bool> = Mutex::new(false);
}

// Logs an error message to a file
fn log_error(message: &str) {
    if let Some(log_file_path) = get_log_file_path() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file_path) {
            let timestamp = current_timestamp();
            if let Err(e) = writeln!(file, "[{}] {}", timestamp, message) {
                eprintln!("Failed to write to log file: {}", e);
            }
        } else {
            eprintln!("Failed to open log file.");
        }
    }
}

// Gets the file path for the log file
fn get_log_file_path() -> Option<PathBuf> {
    home_dir().or_else(|| Some(PathBuf::from("C:\\Users\\Default"))).map(|mut path| {
        path.push(".discord-imhex/error.log");
        path
    })
}

// Converts a string to hex
fn string_to_hex(s: &str) -> String {
    if s == "ImHex" {
        "0".to_string()
    } else {
        s.as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("")
    }
}

// Windows callback function
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let mut title: [u16; 256] = [0; 256];
    let length = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

    if length > 0 {
        let window_title = OsString::from_wide(&title[..length as usize])
            .to_string_lossy()
            .into_owned();

        if window_title.starts_with("ImHex") || window_title.contains("imhex-gui.exe") {
            process_window_title(window_title, lparam);
            return 0;
        }
    }
    1
}

// Processes the window title && logs
fn process_window_title(window_title: String, lparam: LPARAM) {
    let mut previous_title = PREVIOUS_TITLE.lock().unwrap();
    if let Some(index) = window_title.find(" - ") {
        let current_opened_file = &window_title[(index + 3)..];
        if previous_title.as_deref() != Some(current_opened_file) {
            log_error(&format!("Currently opened file: {}", current_opened_file));
            *previous_title = Some(current_opened_file.to_string());
        }
        unsafe { *(lparam as *mut String) = current_opened_file.to_string(); }
    } else {
        if previous_title.as_deref() != Some(&window_title) {
            let hex_string = string_to_hex(&window_title);
            log_error(&format!("Currently opened file: {}", hex_string));
            *previous_title = Some(window_title.clone());
        }
        unsafe { *(lparam as *mut String) = window_title; }
    }
}

// Checks if an ImHex window exists && returns window title
pub fn check_if_imhex_window_exists() -> Option<String> {
    let mut found = String::new();
    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut found as *mut _ as LPARAM);
    }
    if found.is_empty() {
        handle_no_imhex_window();
        None
    } else {
        Some(found)
    }
}

// Handles no ImHex window is found
fn handle_no_imhex_window() {
    let mut previous_title = PREVIOUS_TITLE.lock().unwrap();
    if previous_title.is_some() {
        log_error(&format!("No ImHex window found at {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
        *previous_title = None;
    }
}

// Gets bytes in ImHex
pub fn get_selected_bytes() -> Option<String> {
    if let Some(current_file) = check_if_imhex_window_exists() {
        let hex_string = string_to_hex(&current_file);
        let bytes: Vec<u8> = match hex_string.as_bytes().chunks(2).map(|chunk| {
            u8::from_str_radix(std::str::from_utf8(chunk).unwrap_or_default(), 16).ok()
        }).collect::<Option<Vec<u8>>>() {
            Some(v) => v,
            None => return None,
        };

        if let (Some(&min), Some(&max)) = (bytes.iter().min(), bytes.iter().max()) {
            return Some(format!("0x{:02X}-0x{:02X}", min, max));
        }
    }
    None
}

// Checks if ImHex is running
pub(crate) fn is_imhex_running() -> bool {
    let output = match Command::new("tasklist")
        .arg("/FI")
        .arg("IMAGENAME eq imhex-gui.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .output() 
    {
        Ok(output) => output,
        Err(e) => {
            log_error(&format!("Failed to execute tasklist: {}", e));
            return false;
        }
    };

    let output_str = String::from_utf8_lossy(&output.stdout);
    let is_running = output_str.contains("imhex-gui.exe");
    update_running_state(is_running);
    is_running
}

// Updates the running state
fn update_running_state(is_running: bool) {
    let mut previous_running_state = PREVIOUS_RUNNING_STATE.lock().unwrap();
    if is_running != *previous_running_state {
        log_error(if is_running {
            "ImHex is running."
        } else {
            "ImHex is not running."
        });
        *previous_running_state = is_running;
    }
}
