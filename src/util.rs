
use chrono;

// Produce a formatted string describing the current time.
// Need YYYY/MM/DD HH:MM:SS
pub fn current_time_string() -> String {
    let now = chrono::offset::Local::now();
    format!("{}", now.format("%Y-%m-%d %I:%M:%S %p"))
}
