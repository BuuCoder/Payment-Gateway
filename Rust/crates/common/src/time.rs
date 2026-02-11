use chrono::{DateTime, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn timestamp() -> i64 {
    Utc::now().timestamp()
}
