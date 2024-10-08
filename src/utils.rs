use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Local, DateTime};

pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

pub fn current_timestamp() -> DateTime<Local> {
    Local::now()
}