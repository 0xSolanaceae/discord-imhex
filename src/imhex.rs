use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;

use lazy_static::lazy_static;
use chrono::Local;

use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::HWND;
use winapi::um::winbase::CREATE_NO_WINDOW;
use winapi::um::winuser::{EnumWindows, GetWindowTextW};

lazy_static! {
    static ref PREVIOUS_TITLE: Mutex<Option<String>> = Mutex::new(None);
    static ref PREVIOUS_RUNNING_STATE: Mutex<bool> = Mutex::new(false);
}

fn log_error(message: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("error.log")
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}

fn string_to_hex(s: &str) -> String {
    if s == "ImHex" {
        "0".to_string()
    } else {
        s.as_bytes().iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("")
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let mut title: [u16; 256] = [0; 256];
    let length = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

    if length > 0 {
        let window_title = OsString::from_wide(&title[..length as usize])
            .to_string_lossy()
            .into_owned();

        if window_title.starts_with("ImHex") || window_title.contains("imhex-gui.exe") {
            let mut previous_title = PREVIOUS_TITLE.lock().unwrap();
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            if let Some(index) = window_title.find(" - ") {
                let current_opened_file = &window_title[(index + 3)..];
                if previous_title.as_deref() != Some(current_opened_file) {
                    log_error(&format!("Currently opened file: {} at {}", current_opened_file, timestamp));
                    *previous_title = Some(current_opened_file.to_string());
                }
                *(lparam as *mut String) = current_opened_file.to_string();
            } else {
                if previous_title.as_deref() != Some(&window_title) {
                    let hex_string = string_to_hex(&window_title);
                    log_error(&format!("Currently opened file: {} at {}", hex_string, timestamp));
                    *previous_title = Some(window_title.clone());
                }
                *(lparam as *mut String) = window_title;
            }

            return 0;
        }
    }
    1
}

pub fn check_if_imhex_window_exists() -> Option<String> {
    let mut found = String::new();
    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut found as *mut _ as LPARAM);
    }
    if found.is_empty() {
        let mut previous_title = PREVIOUS_TITLE.lock().unwrap();
        if previous_title.is_some() {
            log_error(&format!("No ImHex window found at {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
            *previous_title = None;
        }
        None
    } else {
        Some(found)
    }
}

pub fn get_selected_bytes() -> Option<String> {
    if let Some(current_file) = check_if_imhex_window_exists() {
        let hex_string = string_to_hex(&current_file);
        let bytes: Vec<u8> = hex_string
            .as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect();
        
        if let (Some(&min), Some(&max)) = (bytes.iter().min(), bytes.iter().max()) {
            return Some(format!("0x{:02X}-0x{:02X}", min, max));
        }
    }
    None
}

pub(crate) fn is_imhex_running() -> bool {
    let output = Command::new("tasklist")
        .arg("/FI")
        .arg("IMAGENAME eq imhex-gui.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .expect("Failed to execute tasklist");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let is_running = output_str.contains("imhex-gui.exe");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut previous_running_state = PREVIOUS_RUNNING_STATE.lock().unwrap();
    if is_running != *previous_running_state {
        if is_running {
            log_error(&format!("ImHex is running at {}", timestamp));
        } else {
            log_error(&format!("ImHex is not running at {}", timestamp));
        }
        *previous_running_state = is_running;
    }
    is_running
}