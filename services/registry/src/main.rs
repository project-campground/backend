use did_method_plc::{Keypair, DIDPLC};
use rocket_db_pools::Database;
use thiserror::Error;
use did_web::DIDWeb;

#[macro_use] extern crate rocket;
extern crate surrealdb_migrations;
extern crate surrealdb;
extern crate thiserror;

pub mod config;
mod well_known;
pub mod xrpc;
mod database;

use database::Registry;

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
    let service_config: config::ServiceConfig = figment.extract_inner("service").expect("service");

    if auth_config.secret_key == "" {
        println!("WARNING: No secret key provided, generating a new one. This is not secure in production!");
        let key = Keypair::generate(did_method_plc::BlessedAlgorithm::K256);
        auth_config.secret_key = key.to_private_key().unwrap();
    }

    let rocket = rocket
        .manage(auth_config.clone())
        .manage(service_config.clone());

    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<(), ProgramError> {
    let rocket = init()?;
    // let rocket = init_db(rocket).await?;

    rocket.launch().await?;

    Ok(())
}