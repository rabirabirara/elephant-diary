
use chrono;


pub const APP_NAME: &'static str = "dated-diary";

// Produce a formatted string describing the current time.
// Need YYYY/MM/DD HH:MM:SS
pub fn current_time_string() -> String {
    let now = chrono::offset::Local::now();
    format!("{}", now.format("%Y-%m-%d %I:%M:%S %p"))
}

