#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::{util::is_s32, TID};

#[derive(Clone, Debug)]
pub enum RecordKey {
    TID(TID),
    Literal(String),
    Any(String),
    Invalid
}

impl Ord for RecordKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl PartialOrd for RecordKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RecordKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for RecordKey {}

impl ToString for RecordKey {
    fn to_string(&self) -> String {
        match self {
            RecordKey::TID(tid) => {
                tid.to_string()
            },
            RecordKey::Literal(s) => {
                format!("literal:{}", s)
            },
            RecordKey::Any(s) => {
                s.to_string()
            },
            RecordKey::Invalid => {
                "".to_string()
            }
        }
    }
}

impl From<&str> for RecordKey {
    fn from(s: &str) -> Self {
        RecordKey::from(s.to_string())
    }
}

impl From<String> for RecordKey {
    fn from(s: String) -> Self {
        let no_dashes = s.replace("-", "");
        if no_dashes.len() == 13 && is_s32(&no_dashes) {
            let len = no_dashes.len();
            if !no_dashes.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ':' || c == '~') {
                return RecordKey::Invalid
            }
            if len < 1 || len > 512 {
                return RecordKey::Invalid
            }
            if no_dashes == "." || no_dashes == ".." {
                return RecordKey::Invalid
            }


            return RecordKey::TID(TID::from(no_dashes))
        }

        if s.starts_with("literal:") {
            let res = s[8..].to_string();
            
            let len = res.len();
            if !res.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ':' || c == '~') {
                return RecordKey::Invalid
            }
            if len < 1 || len > 512 {
                return RecordKey::Invalid
            }
            if res == "." || res == ".." {
                return RecordKey::Invalid
            }

            RecordKey::Literal(res)
        } else {
            let len = s.len();
            if !s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ':' || c == '~') {
                return RecordKey::Invalid
            }
            if len < 1 || len > 512 {
                return RecordKey::Invalid
            }
            if s == "." || s == ".." {
                return RecordKey::Invalid
            }

            RecordKey::Any(s)
        }
    }
}

impl Serialize for RecordKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(RecordKey::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rk_type(key: &RecordKey) -> String {
        match key {
            RecordKey::TID{..} => "TID".to_string(),
            RecordKey::Literal(..) => "Literal".to_string(),
            RecordKey::Any(..) => "Any".to_string(),
            RecordKey::Invalid => "Invalid".to_string(),
        }
    }

    #[test]
    fn test_record_tid() {
        // Valid TIDs should properly decode
        let tid = RecordKey::from("3jzfcijpj2z2a");
        assert_eq!(rk_type(&tid), "TID");
        assert_eq!(tid.to_string(), "3jzfcijpj2z2a");

        let tid = RecordKey::from("7777777777777");
        assert_eq!(rk_type(&tid), "TID");
        assert_eq!(tid.to_string(), "7777777777777");

        let tid = RecordKey::from("3zzzzzzzzzzzz");
        assert_eq!(rk_type(&tid), "TID");
        assert_eq!(tid.to_string(), "3zzzzzzzzzzzz");

        let tid = RecordKey::from("2222222222222");
        assert_eq!(rk_type(&tid), "TID");
        assert_eq!(tid.to_string(), "2222222222222");

        // legacy dash syntax should properly decode
        let tid = RecordKey::from("3jzf-cij-pj2z-2a");
        assert_eq!(rk_type(&tid), "TID");
        assert_eq!(tid.to_string(), "3jzfcijpj2z2a");
    }

    #[test]
    fn test_record_literal() {
        let lit = RecordKey::from("literal:self");
        assert_eq!(rk_type(&lit), "Literal");
        assert_eq!(lit.to_string(), "literal:self"); 
    }

    #[test]
    fn test_record_any() {
        // Test valid key
        let any = RecordKey::from("self");
        assert_eq!(rk_type(&any), "Any");
        assert_eq!(any.to_string(), "self");

        // Test invalid keys
        let any = RecordKey::from("self?");
        assert_eq!(rk_type(&any), "Invalid");

        let any = RecordKey::from("");
        assert_eq!(rk_type(&any), "Invalid");

        let any = RecordKey::from("..");
        assert_eq!(rk_type(&any), "Invalid");

        let any = RecordKey::from("alpha/beta");
        assert_eq!(rk_type(&any), "Invalid");
    }
}