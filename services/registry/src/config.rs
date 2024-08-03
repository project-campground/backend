use rocket::serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DBConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub namespace: String,
}

impl Default for DBConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8000,
            user: "root".to_string(),
            password: "example".to_string(),
            database: "registry".to_string(),
            namespace: "campground.gg".to_string(),
        }
    }
}

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