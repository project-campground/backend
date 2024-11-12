#![deny(unsafe_code)]
#![warn(
    clippy::cognitive_complexity,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::float_equality_without_abs,
    keyword_idents,
    clippy::missing_const_for_fn,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::mod_module_files,
    non_ascii_idents,
    noop_method_call,
    clippy::option_if_let_else,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::semicolon_if_nothing_returned,
    clippy::unseparated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    clippy::unused_self,
    clippy::use_debug,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq,
    clippy::wildcard_dependencies,
    clippy::wildcard_imports
)]

use std::env;
use rand::Rng;
use reqwest as _;

use did_method_plc::{Keypair, DIDPLC};
use thiserror::Error;
use did_web::DIDWeb;

#[macro_use] extern crate rocket;

pub mod config;
mod well_known;
mod repository;
pub mod schema;
pub mod xrpc;
mod database;
mod context;

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Database error")]
    DBError(#[from] diesel::ConnectionError),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
}

pub fn init() -> Result<rocket::Rocket<rocket::Build>, ProgramError> {
    let didplc = DIDPLC::default();
    let didweb = DIDWeb {};

    let rocket = rocket::build()
        .mount("/", routes![
        .mount("/", routes![])
        .mount("/.well-known", well_known::routes())
        .register("/", catchers![default_catcher])
        .manage(didplc)
        .manage(didweb);

    let figment = rocket.figment();

    let mut auth_config: config::AuthConfig = figment.extract_inner("auth").expect("auth");
    let mut service_config: config::ServiceConfig = figment.extract_inner("service").expect("service");

    if auth_config.secret_key == "" {
        println!("WARNING: No auth secret key provided, generating a new one. This is not secure in production!");
        let key = Keypair::generate(did_method_plc::BlessedAlgorithm::K256);
        auth_config.secret_key = key.to_private_key().unwrap();
    }

    if service_config.secret_key == "" {
        println!("WARNING: No service secret key provided, generating a new one. This is not secure in production!");
        let key = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();
        service_config.secret_key = key;
    }

    let rocket = rocket
        .manage(auth_config.clone())
        .manage(service_config.clone());

    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<(), ProgramError> {
    let rocket = init()?;

    rocket.launch().await?;

    Ok(())
}