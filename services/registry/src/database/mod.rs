use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use rocket::Config;
use crate::config::DatabaseConfig;
use diesel::{pg::PgConnection, r2d2::{Pool, ConnectionManager, PooledConnection}};
use lazy_static::lazy_static;

pub mod models;
pub use self::models::*;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref CONFIG: DatabaseConfig = Config::figment()
        .extract_inner("database")
        .expect("Failed to load database configuration");

    static ref POOL: OnceLock<Mutex<DbPool>> = OnceLock::new();
}

pub fn establish_connection() -> Result<DbConnection> {
    let pool = POOL.get_or_init(|| {
        let manager = ConnectionManager::<PgConnection>::new(&CONFIG.url);
        Mutex::new(
            Pool::builder()
                .max_size(CONFIG.pool_size.clone())
                .build(manager)
                .expect("Failed to create connection pool")
            )
    });
    Ok(pool.lock().unwrap().get()?)
}