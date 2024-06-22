use rocket::serde::Deserialize;

#[derive(Debug, Deserialize)]
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