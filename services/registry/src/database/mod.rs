use rocket::http::Status;
use rocket_db_pools::{Database, Connection};
use surrealdb::{engine::any::Any, Surreal};
use surrealdb_migrations::MigrationRunner;
use deadpool_surrealdb::SurrealDBPool;
use include_dir::include_dir;

mod models;

#[allow(unused_imports)]
pub use models::*;

use crate::internal::InternalEndpoint;

#[derive(Database)]
#[database("registry")]
#[allow(missing_debug_implementations)]
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

#[allow(dead_code)]
pub async fn migrate_test_db(db: &Surreal<Any>) -> bool {
    let res = MigrationRunner::new(db)
        .load_files(&include_dir!("$CARGO_MANIFEST_DIR/db"))
        .up()
        .await;

    match res {
        Ok(_) => true,
        Err(_) => false
    }
}

#[post("/migrate-db")]
async fn migrate_db_route(db: Connection<Registry>, _auth: InternalEndpoint) -> Result<String, Status> {
    if migrate_db(&db).await {
        Ok("OK".to_string())
    } else {
        Err(Status::InternalServerError)
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![migrate_db_route]
}