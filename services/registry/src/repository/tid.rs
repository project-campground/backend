use serde::{Deserialize, Serialize};

use super::util::{is_s32, s32decode, s32encode};

pub struct TIDGenerator {
    last_timestamp: f64,
    timestamp_count: i32,
    clockid: f64,
}

impl TIDGenerator {
    pub fn new() -> Self {
        let clockid = (rand::random::<i64>() * 32) as f64;
        Self {
            last_timestamp: 0.0,
            timestamp_count: 0,
            clockid,
        }
    }

    pub fn next(&mut self, prev: Option<TID>) -> TID {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
        if now == self.last_timestamp {
            self.timestamp_count += 1;
        }
        self.last_timestamp = now;
        let timestamp = now * 1000.0 + self.timestamp_count as f64;
        let tid = TID::new(timestamp, self.clockid);
        if prev.is_none() {
            tid
        } else {
            let prev = prev.unwrap();
            if tid > prev {
                return tid;
            }
            TID::new(timestamp + 1.0, self.clockid)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord)]
pub struct TID(String);

impl TID {
    pub fn new(timestamp: f64, clockid: f64) -> Self {
        if timestamp == 0.0 && clockid == 0.0 {
            return Self("2222222222222".to_string())
        }
        let timestamp = s32encode(timestamp);
        let clockid = s32encode(clockid);
        let clockid = if clockid.len() < 2 {
            format!("2{}", clockid)
        } else {
            clockid
        };
        Self(format!("{}{}", timestamp, clockid))
    }

    pub fn timestamp(&self) -> f64 {
        s32decode(self.0[0..11].to_string())
    }
    
    pub fn clockid(&self) -> f64 {
        s32decode(self.0[11..13].to_string())
    }
}

impl PartialEq for TID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for TID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl ToString for TID {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for TID {
    fn from(s: String) -> Self {
        let no_dashes = s.replace("-", "");
        if no_dashes.len() != 13 || !is_s32(&no_dashes) {
            panic!("Invalid TID string: {}", s);
        }
        Self(no_dashes)
    }
}

impl From<&str> for TID {
    fn from(s: &str) -> Self {
        TID::from(s.to_string())
    }
}