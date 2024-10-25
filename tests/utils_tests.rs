#[path = "../src/utils.rs"]
mod utils;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use chrono::DateTime;
    use regex::Regex;
    use utils::{current_timestamp, get_current_timestamp};

    #[test]
    fn test_get_current_timestamp() {
        let timestamp1 = get_current_timestamp();
        thread::sleep(Duration::from_secs(1));
        let timestamp2 = get_current_timestamp();
        
        assert!(timestamp1 > 0);
        
        assert!(timestamp2 >= timestamp1);
        
        assert!(timestamp2 - timestamp1 >= 1);
        assert!(timestamp2 - timestamp1 <= 2);
    }

    #[test]
    fn test_current_timestamp_format() {
        let timestamp = current_timestamp();
        
        // Regex pattern for "YYYY-MM-DD HH:MM:SS" format
        let pattern = Regex::new(
            r"^\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01]) (?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d$"
        ).unwrap();
        
        assert!(pattern.is_match(&timestamp));
    }

    #[test]
    fn test_current_timestamp_components() {
        let timestamp = current_timestamp();
        let parts: Vec<&str> = timestamp.split(' ').collect();
        
        // Should have date and time parts
        assert_eq!(parts.len(), 2);
        
        // Check date components
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3);
        
        // Year should be current year (you might want to adjust this range)
        let year: i32 = date_parts[0].parse().unwrap();
        assert!(year >= 2024 && year <= 2100);
        
        // Month should be 1-12
        let month: i32 = date_parts[1].parse().unwrap();
        assert!(month >= 1 && month <= 12);
        
        // Day should be 1-31
        let day: i32 = date_parts[2].parse().unwrap();
        assert!(day >= 1 && day <= 31);
        
        // Check time components
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        assert_eq!(time_parts.len(), 3);
        
        // Hour should be 0-23
        let hour: i32 = time_parts[0].parse().unwrap();
        assert!(hour >= 0 && hour <= 23);
        
        // Minutes should be 0-59
        let minutes: i32 = time_parts[1].parse().unwrap();
        assert!(minutes >= 0 && minutes <= 59);
        
        // Seconds should be 0-59
        let seconds: i32 = time_parts[2].parse().unwrap();
        assert!(seconds >= 0 && seconds <= 59);
    }

    fn create_timestamp() -> String {
        current_timestamp()
    }

    fn parse_timestamp(timestamp: &str) -> DateTime<chrono::FixedOffset> {
        DateTime::parse_from_str(&format!("{} +0000", timestamp), "%Y-%m-%d %H:%M:%S %z").unwrap()
    }
    #[test]
    fn test_sequential_timestamps() {
        let timestamp1 = create_timestamp();
        thread::sleep(Duration::from_secs(1));
        let timestamp2 = create_timestamp();
    
        let dt1 = parse_timestamp(&timestamp1);
        let dt2 = parse_timestamp(&timestamp2);
    
        assert!(dt2 > dt1);
    }
}