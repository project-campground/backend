use surreal_bb8::temp::{config::Config, runtime_with_config::SurrealConnectionManager};
// use surrealdb_migrations::MigrationRunner;
use rocket_db_pools::Database;
use did_method_plc::{Keypair, DIDPLC};
use deadpool_surrealdb::SurrealDBPool;
// use include_dir::include_dir;
use surrealdb::opt::auth::Root;
use surreal_bb8::bb8::Pool;
use thiserror::Error;
use did_web::DIDWeb;

#[macro_use] extern crate rocket;
extern crate surrealdb_migrations;
extern crate surrealdb;
extern crate thiserror;

pub mod config;
mod well_known;
pub mod xrpc;

#[derive(Database)]
#[database("registry")]
struct Registry(SurrealDBPool);

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Database error")]
    DBError(#[from] surrealdb::Error),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
}

pub fn init() -> Result<rocket::Rocket<rocket::Build>, ProgramError> {
    let didplc = DIDPLC::default();
    let didweb = DIDWeb {};

    let rocket = rocket::build()
        .attach(Registry::init())
        .mount("/", routes![])
        .mount("/.well-known", well_known::routes())
        .manage(didplc)
        .manage(didweb);

    let figment = rocket.figment();

    let mut auth_config: config::AuthConfig = figment.extract_inner("auth").expect("auth");
    //let db_config: config::DBConfig = figment.extract_inner("surreal").expect("host");
    let service_config: config::ServiceConfig = figment.extract_inner("service").expect("service");

    if auth_config.secret_key == "" {
        println!("WARNING: No secret key provided, generating a new one. This is not secure in production!");
        let key = Keypair::generate(did_method_plc::BlessedAlgorithm::K256);
        auth_config.secret_key = key.to_private_key().unwrap();
    }

    let rocket = rocket
        .manage(auth_config.clone())
    //    .manage(db_config.clone())
        .manage(service_config.clone());

    Ok(rocket)
}

pub async fn init_db(rocket: rocket::Rocket<rocket::Build>) -> Result<rocket::Rocket<rocket::Build>, ProgramError> {
    let figment = rocket.figment();
    let db_config: config::DBConfig = figment.extract_inner("surreal").expect("host");

    let sur_mgr = SurrealConnectionManager::new(
        Config::new()
            .user(Root {
                username: &db_config.user,
                password: &db_config.password,
            }),
        format!("{}:{}", db_config.host, db_config.port).as_str()
    );
    let pool = Pool::builder().build(sur_mgr).await.expect("build error");
    let rocket = rocket.manage(pool);

    // let connection = pool.get().await.expect("pool error");
    // connection.use_ns(&db_config.namespace).use_db(&db_config.database).await?;

    // let _ = MigrationRunner::new(&connection)
    //     .load_files(&include_dir!("$CARGO_MANIFEST_DIR/db"))
    //     .up()
    //     .await;
        //.expect("Failed to apply migrations");
    
    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<(), ProgramError> {
    let rocket = init()?;
    // let rocket = init_db(rocket).await?;

    rocket.launch().await?;

    Ok(())
}