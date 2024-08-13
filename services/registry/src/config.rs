use rocket::serde::Deserialize;
use did_method_plc::Keypair;

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AuthConfig {
    pub secret_key: String,
}

impl AuthConfig {
    pub fn secret_key(&self) -> Keypair {
        Keypair::from_private_key(&self.secret_key).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ServiceConfig {
    pub public_url: String,
    pub did: String,
    pub secret_key: String,
}

impl ServiceConfig {
    pub fn secret_key(&self) -> Keypair {
        Keypair::from_private_key(&self.secret_key).unwrap()
    }
}