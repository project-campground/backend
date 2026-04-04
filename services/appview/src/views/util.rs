use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};

pub fn parse_datetime(datetime: Option<DateTime<Utc>>) -> Option<String> {
    match datetime {
        Some(dt) => Some(dt.to_rfc3339()),
        None => None,
    }
}

pub fn serialize_datetime(datetime: NaiveDateTime) -> String {
    datetime.format("%FT%H:%M:%S%.3fZ").to_string()
}
