use std::{env, sync::{Mutex, OnceLock}};

use anyhow::Result;
use diesel::{pg::PgConnection, r2d2::{Pool, ConnectionManager, PooledConnection}};
use lazy_static::lazy_static;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref POOL: OnceLock<Mutex<DbPool>> = OnceLock::new();
}

pub fn establish_connection() -> Result<DbConnection> {
    let pool = POOL.get_or_init(|| {
        let manager = ConnectionManager::<PgConnection>::new(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
        Mutex::new(
            Pool::builder()
                .max_size(env::var("DB_POOL_SIZE").unwrap_or("10".to_string()).parse::<u32>().unwrap())
                .build(manager)
                .expect("Failed to create connection pool")
            )
    });
    Ok(pool.lock().unwrap().get()?)
}

pub use appview_schema::models;