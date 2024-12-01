use std::collections::HashMap;
use crate::{operation::{
    PLCOperation, Service, SignedGenesisOperation, SignedPLCOperation
}, PLCError};

pub fn op_from_json(s: &str) -> Result<PLCOperation, PLCError> {
    let json: serde_json::Value = serde_json::from_str(s).unwrap();
    let op = json.as_object().unwrap().clone();

    if op.get("sig").is_none() {
        return Err(PLCError::InvalidSignature)
    }

    match op.get("type").unwrap().as_str().unwrap() {
        "plc_operation" => {
            let op = SignedPLCOperation::from_json(s)?;
            Ok(PLCOperation::SignedPLC(op))
        },
        "plc_tombstone" => {
            let op = SignedPLCOperation::from_json(s)?;
            Ok(PLCOperation::SignedPLC(op))
        },
        "create" => {
            let op = SignedGenesisOperation::from_json(s)?;
            Ok(PLCOperation::SignedGenesis(op))
        },
        _ => Err(PLCError::InvalidOperation)
    }
}

pub fn assure_at_prefix(s: &str) -> String {
    if s.starts_with("at://") {
        s.to_string()
    } else {
        format!("at://{}", s)
    }
}

pub fn assure_http(s: &str) -> String {
    if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("https://{}", s)
    }
}

pub fn normalize_op(json: serde_json::Value) -> serde_json::Value {
    let json = json.as_object().unwrap().clone();
    let mut normalized_json = serde_json::Map::new();

    if json.get("type").unwrap() == "create" {
        // This is a legacy genesis operation format
        let mut rotation_keys: Vec<String> = vec![];
        let mut verification_methods: HashMap<String, String> = HashMap::new();
        let mut also_known_as: Vec<String> = vec![];
        let mut services: HashMap<String, Service> = HashMap::new();

        if !json.get("recoveryKey").unwrap().is_null() {
            rotation_keys.push(json.get("recoveryKey").unwrap().to_string());
        }
        if !json.get("signingKey").unwrap().is_null() {
            let key = json.get("signingKey").unwrap().to_string();
            rotation_keys.push(key.clone());
            verification_methods.insert("atproto".to_string(), key);
        }
        if !json.get("handle").unwrap().is_null() {
            also_known_as.push(assure_at_prefix(
                json.get("handle").unwrap().as_str().unwrap(),
            ));
        }
        if !json.get("service").unwrap().is_null() {
            services.insert(
                "atproto_pds".to_string(),
                Service {
                    type_: "AtprotoPersonalDataServer".to_string(),
                    endpoint: assure_http(
                        json.get("service")
                            .unwrap()
                            .get("endpoint")
                            .unwrap()
                            .as_str()
                            .unwrap(),
                    ),
                },
            );
        }
        normalized_json.insert(
            "type".to_string(),
            serde_json::Value::String("plc_operation".to_string()),
        );
        normalized_json.insert("prev".to_string(), serde_json::Value::Null);
        normalized_json.insert(
            "rotationKeys".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                rotation_keys
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
        normalized_json.insert(
            "verificationMethods".to_string(),
            serde_json::to_value(verification_methods).unwrap(),
        );
        normalized_json.insert(
            "alsoKnownAs".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                also_known_as
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
        normalized_json.insert(
            "services".to_string(),
            serde_json::to_value(services).unwrap(),
        );
    } else {
        for (key, value) in json.iter() {
            normalized_json.insert(key.clone(), value.clone());
        }
    }

    if normalized_json.get("alsoKnownAs").is_some() && !normalized_json.get("alsoKnownAs").unwrap().is_null() {
        let mut also_known_as = vec![];
        for value in normalized_json
            .get("alsoKnownAs")
            .unwrap()
            .as_array()
            .unwrap()
        {
            also_known_as.push(assure_at_prefix(value.as_str().unwrap()));
        }
        normalized_json.insert(
            "alsoKnownAs".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                also_known_as
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
    }

    serde_json::to_value(normalized_json).unwrap()
}