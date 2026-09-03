use std::sync::{Mutex, OnceLock};

use crate::config::DATABASE_CONFIG;
use anyhow::Result;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use lazy_static::lazy_static;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref POOL: OnceLock<Mutex<DbPool>> = OnceLock::new();
}

pub fn establish_connection() -> Result<DbConnection> {
    let pool = POOL.get_or_init(|| {
        let manager = ConnectionManager::<PgConnection>::new(&DATABASE_CONFIG.url);
        Mutex::new(
            Pool::builder()
                .max_size(DATABASE_CONFIG.pool_size.clone())
                .build(manager)
                .expect("Failed to create connection pool"),
        )
    });
    Ok(pool.lock().unwrap().get()?)
}

pub mod actors;
pub mod campsites;
pub mod profile_posts;
pub mod profiles;
pub use appview_schema::models;
pub mod messages;
