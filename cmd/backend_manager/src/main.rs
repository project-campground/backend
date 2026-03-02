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

use std::io;

use anyhow::Result;
use lazy_static::lazy_static;

use crate::{actions::setup_actor::setup_actor, util::prompt::read_stdin};

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

async fn menu(database_url: &str, pds_port: u16) -> Result<(), String> {
    println!("\
        =======================\n\
        0. Exit\n\
        1. Setup actor/account\n\
    ");

    let mut stdin = io::stdin();
    let mut input = String::new();
    read_stdin(&mut stdin, &mut input, "menu item index")?;

    match input.as_str() {
        "0" => {
            return Ok(());
        },
        "1" => {
            setup_actor(&HTTP_CLIENT, database_url, pds_port).await?;
        },
        _ => {
            println!("Unknown option {:?}", input);
        }
    };
    return Ok(Box::pin(menu(database_url, pds_port)).await?);
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    println!("Started");
    let mut stdin = io::stdin();
    let mut database_url = String::new();
    read_stdin(&mut stdin, &mut database_url, "PostgreSQL database URL (likely from Rocket.toml in services/appview that you have edited)")
        .expect("Error reading STDIN");
    let mut pds_port = String::new();
    read_stdin(&mut stdin, &mut pds_port, "port for the PDS (default 2583 in Rocket.toml config)")
        .expect("Error reading STDIN");
    let pds_port: u16 = pds_port.parse().expect("Expected an unsigned integer as a PDS port");

    println!("PDS port: {}, Database URL: {:?}", pds_port, database_url);
    // let mut conn = establish_connection().unwrap();
    let result = menu(database_url.as_str(), pds_port)
        .await;

    if let Err(err) = result {
        println!("Error: {}", err);
    }
}

mod database;
mod actions;
mod util;
pub use appview_schema::schema;