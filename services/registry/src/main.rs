use surreal_bb8::temp::{config::Config, runtime_with_config::SurrealConnectionManager};
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;
use surreal_bb8::bb8::Pool;
use thiserror::Error;

#[macro_use] extern crate rocket;
extern crate surrealdb_migrations;
extern crate surrealdb;
extern crate thiserror;

pub mod config;

#[derive(Error, Debug)]
enum ProgramError {
    #[error("Database error")]
    DBError(#[from] surrealdb::Error),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
}

#[rocket::main]
async fn main() -> Result<(), ProgramError> {
    let rocket = rocket::build()
        .mount("/", routes![])
        .launch()
        .await?;
    let figment = rocket.figment();

    let db_config: config::DBConfig = figment.extract_inner("db").expect("custom");

    let sur_mgr = SurrealConnectionManager::new(
        Config::new()
            .user(Root {
                username: &db_config.user,
                password: &db_config.password,
            }),
        format!("{}:{}", db_config.host, db_config.port).as_str()
    );
    let pool = Pool::builder().build(sur_mgr).await.expect("build error");

    let connection = pool.get().await.expect("pool error");
    connection.use_ns(&db_config.namespace).use_db(&db_config.database).await?;

    MigrationRunner::new(&connection)
        .up()
        .await
        .expect("Failed to apply migrations");

    Ok(())
}