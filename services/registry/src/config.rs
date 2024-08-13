use rocket::serde::Deserialize;


#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AuthConfig {
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ServiceConfig {
    pub public_url: String,
    pub did: String,
}