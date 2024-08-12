use rocket_db_pools::{Database, Connection};
use surrealdb_migrations::MigrationRunner;
use deadpool_surrealdb::SurrealDBPool;
use include_dir::include_dir;

mod models;

#[allow(unused_imports)]
pub use models::*;

#[derive(Database)]
#[database("registry")]
pub struct Registry(SurrealDBPool);

#[allow(dead_code)]
pub async fn migrate_db(db: &Connection<Registry>) -> bool {
    let res = MigrationRunner::new(db)
        .load_files(&include_dir!("$CARGO_MANIFEST_DIR/db"))
        .up()
        .await;

    match res {
        Ok(_) => true,
        Err(_) => false
    }
}