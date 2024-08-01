use serde::{Deserialize, Serialize};
use did_method_plc::Keypair;
use base64::Engine;

#[derive(thiserror::Error, Debug)]
pub enum JWTError {
    #[error("Invalid JWT")]
    Invalid,
    #[error("Invalid algorithm")]
    InvalidAlgorithm,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid keypair")]
    InvalidKeypair,
}

#[derive(Debug)]
pub enum JWTAlgorithm {
    P256,
    K256,
}

impl ToString for JWTAlgorithm {
    fn to_string(&self) -> String {
        match self {
            JWTAlgorithm::P256 => "ES256".to_string(),
            JWTAlgorithm::K256 => "ES256K".to_string(),
        }
    }
}

impl JWTAlgorithm {
    pub fn from_str(alg: &str) -> Option<Self> {
        match alg {
            "ES256" => Some(JWTAlgorithm::P256),
            "ES256K" => Some(JWTAlgorithm::K256),
            _ => None,
        }
    }
}

impl Default for JWTAlgorithm {
    fn default() -> Self {
        JWTAlgorithm::P256
    }
}

impl Serialize for JWTAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for JWTAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        JWTAlgorithm::from_str(&s).ok_or_else(|| serde::de::Error::custom("Invalid algorithm"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWT {
    #[serde(rename = "alg")]
    pub algorithm: JWTAlgorithm,
    #[serde(rename = "typ")]
    pub jwt_type: String,
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "exp")]
    pub expiration_time: Option<u64>,
    #[serde(rename = "aud")]
    pub audience: Option<String>,
    #[serde(rename = "sub")]
    pub subject: Option<String>,
    #[serde(rename = "sig")]
    pub signature: Option<Vec<u8>>,
}

impl JWT {
    pub fn from_str(token: &str) -> Result<Self, JWTError> {
        let parts = token.split('.').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(JWTError::Invalid);
        }
        let engine = base64::engine::general_purpose::URL_SAFE;
        let header = engine.decode(parts[0]).map_err(|_| JWTError::Invalid)?;
        let payload = engine.decode(parts[1]).map_err(|_| JWTError::Invalid)?;
        let signature = engine.decode(parts[2]).map_err(|_| JWTError::Invalid)?;

        let header_values =
            serde_json::from_slice::<serde_json::Value>(&header).map_err(|_| JWTError::Invalid)?;
        let payload_values =
            serde_json::from_slice::<serde_json::Value>(&payload).map_err(|_| JWTError::Invalid)?;

        let algorithm = header_values
            .get("alg")
            .and_then(|v| JWTAlgorithm::from_str(v.as_str().unwrap_or("")))
            .ok_or(JWTError::InvalidAlgorithm)?;
        let jwt_type = header_values
            .get("typ")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let issuer = payload_values
            .get("iss")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(JWTError::Invalid)?;
        let expiration_time = payload_values
            .get("exp")
            .and_then(|v| v.as_u64());
        let audience = payload_values
            .get("aud")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let subject = payload_values
            .get("sub")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(JWT {
            algorithm,
            jwt_type,
            issuer,
            expiration_time,
            audience,
            subject,
            signature: Some(signature),
        })
    }

    pub fn sign(&mut self, key: &Keypair) -> Result<(), JWTError> {
        let engine = base64::engine::general_purpose::URL_SAFE;
        let header = serde_json::json!({
            "alg": self.algorithm.to_string(),
            "typ": self.jwt_type,
        });
        let payload = serde_json::json!({
            "iss": self.issuer,
            "exp": self.expiration_time,
            "aud": self.audience,
            "sub": self.subject,
        });
        let msg = format!("{}.{}", engine.encode(header.to_string()), engine.encode(payload.to_string()));
        let signature = key.sign(msg.as_bytes()).map_err(|_| JWTError::InvalidKeypair)?;
        self.signature = Some(signature);
        Ok(())
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        assert!(self.signature.is_some(), "JWT is missing a signature!");
        let engine = base64::engine::general_purpose::URL_SAFE;
        let header = serde_json::json!({
            "alg": self.algorithm.to_string(),
            "typ": self.jwt_type,
        });
        let payload = serde_json::json!({
            "iss": self.issuer,
            "exp": self.expiration_time,
            "aud": self.audience,
            "sub": self.subject,
        });
        let msg = format!("{}.{}", engine.encode(header.to_string()), engine.encode(payload.to_string()));
        let signature = self.signature.as_ref().unwrap();
        format!("{}.{}", msg, engine.encode(signature))
    }
}
