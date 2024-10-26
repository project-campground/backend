use anyhow::Result;
use rocket::figment::providers::Format;
use crate::config::DatabaseConfig;
use rocket::figment::{Figment, providers::Toml};
use diesel::{pg::PgConnection, Connection};
use lazy_static::lazy_static;

pub mod models;
pub use self::models::*;

lazy_static! {
    static ref CONFIG: DatabaseConfig = Figment::new()
        .merge(Toml::file("Rocket.toml"))
        .extract_inner("default.database")
        .expect("Failed to load database configuration");
}

pub fn establish_connection() -> Result<PgConnection> {
    Ok(PgConnection::establish(&CONFIG.url).map_err(|error| {
        let context = format!("Error connecting to {:?}", CONFIG.url);
        anyhow::Error::new(error).context(context)
    })?)
}