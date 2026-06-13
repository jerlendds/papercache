use chrono::Utc;
use std::time::SystemTime;

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn system_time_rfc3339(time: SystemTime) -> String {
    chrono::DateTime::<Utc>::from(time).to_rfc3339()
}
