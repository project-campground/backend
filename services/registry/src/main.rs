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
use api::bsky_api_forwarder;
use rocket::shield::{NoSniff, Shield};
use rsky_identity::types::{DidCache, IdentityResolverOpts};
use rsky_identity::IdResolver;
use tokio::sync::RwLock;
use database::establish_connection;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::response::status;
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Request, Response};
use diesel::prelude::*;
use diesel::sql_types::Int4;
use reqwest as _;
use anyhow::Result;
use rsky_pds::crawlers::Crawlers;
use rsky_pds::SharedIdResolver;
use crate::sequencer::Sequencer;
use crate::config::{IDENTITY_CONFIG, CORE_CONFIG};
use event_emitter_rs::EventEmitter;
use lazy_static::lazy_static;

#[macro_use] extern crate rocket;

mod account_manager;
mod auth_verifier;
mod pipethrough;
mod well_known;
mod repository;
mod sequencer;
mod database;
mod context;
mod config;
mod schema;
mod xrpc;
mod api;

pub const INVALID_HANDLE: &'static str = "handle.invalid";
pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug)]
pub struct SharedSequencer {
    pub sequencer: RwLock<Sequencer>,
}

// Use lazy_static! because the size of EventEmitter is not known at compile time
lazy_static! {
    // Export the emitter with `pub` keyword
    pub static ref EVENT_EMITTER: RwLock<EventEmitter> = RwLock::new(EventEmitter::new());
}

struct CORS;

#[get("/robots.txt")]
async fn robots() -> &'static str {
    "# Hello!\n\n# Crawling the public API is allowed\nUser-agent: *\nAllow: /"
}

#[get("/xrpc/_health")]
async fn health() -> Result<
    Json<rsky_pds::models::ServerVersion>,
    status::Custom<Json<rsky_pds::models::ErrorMessageResponse>>,
> {
    let conn = &mut match establish_connection() {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Internal Error: {error}");
            let internal_error = rsky_pds::models::ErrorMessageResponse {
                code: Some(rsky_pds::models::ErrorCode::ServiceUnavailable),
                message: Some(error.to_string()),
            };
            return Err(status::Custom(
                Status::ServiceUnavailable,
                Json(internal_error),
            ));
        }
    };
    let result =
        diesel::select(diesel::dsl::sql::<Int4>("1")) // SELECT 1;
            .load::<i32>(conn)
            .map(|v| v.into_iter().next().expect("no results"));
    match result {
        Ok(_) => {
            let env_version = env!("CARGO_PKG_VERSION").to_owned();
            let version = rsky_pds::models::ServerVersion {
                version: env_version,
            };
            Ok(Json(version))
        }
        Err(error) => {
            eprintln!("Internal Error: {error}");
            let internal_error = rsky_pds::models::ErrorMessageResponse {
                code: Some(rsky_pds::models::ErrorCode::ServiceUnavailable),
                message: Some(error.to_string()),
            };
            Err(status::Custom(
                Status::ServiceUnavailable,
                Json(internal_error),
            ))
        }
    }
}

#[catch(default)]
async fn default_catcher() -> Json<rsky_pds::models::ErrorMessageResponse> {
    let internal_error = rsky_pds::models::ErrorMessageResponse {
        code: Some(rsky_pds::models::ErrorCode::InternalServerError),
        message: Some("Internal error.".to_string()),
    };
    Json(internal_error)
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
async fn all_options() {
    /* Intentionally left empty */
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS, DELETE",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub async fn init() -> Result<rocket::Rocket<rocket::Build>> {
    let sequencer = SharedSequencer {
        sequencer: RwLock::new(Sequencer::new(
            Crawlers::new(CORE_CONFIG.hostname(), CORE_CONFIG.crawlers.clone()),
            None,
        )),
    };
    let mut background_sequencer = sequencer.sequencer.write().await.clone();
    tokio::spawn(async move { background_sequencer.start().await });

    let aws_sdk_config = aws_config::from_env()
        .endpoint_url(CORE_CONFIG.aws_endpoint.clone().unwrap_or("localhost".to_owned()))
        .load()
        .await;

    let id_resolver = SharedIdResolver {
        id_resolver: RwLock::new(IdResolver::new(IdentityResolverOpts {
            timeout: None,
            plc_url: Some(IDENTITY_CONFIG.plc_url.clone()),
            did_cache: Some(DidCache::new(None, None)),
            backup_nameservers: IDENTITY_CONFIG.handle_backup_name_servers.clone()
        })),
    };

    let shield = Shield::default().enable(NoSniff::Enable);

    let rocket = rocket::build()
        .mount("/", routes![
            api::com::atproto::identity::resolve_handle::resolve_handle,
            api::com::atproto::identity::update_handle::update_handle,
            api::com::atproto::server::create_account::create_account,
            api::com::atproto::server::create_session::create_session,
            api::com::atproto::server::delete_session::delete_session,
            api::com::atproto::server::get_session::get_session,
            api::com::atproto::server::refresh_session::refresh_session,
            robots,
            health,
            bsky_api_forwarder,
            all_options
        ])
        .mount("/.well-known", well_known::routes())
        .register("/", catchers![default_catcher])
        .attach(shield)
        .attach(CORS)
        .manage(sequencer)
        .manage(aws_sdk_config)
        .manage(id_resolver);

    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = init().await?;

    rocket.launch().await?;

    Ok(())
}