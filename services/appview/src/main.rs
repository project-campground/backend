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

use rocket::fairing::{Fairing, Info, Kind};
use rocket::shield::{Shield, NoSniff};
use rocket::{Request, Response};
use rsky_identity::types::{DidCache, IdentityResolverOpts};
use rsky_identity::IdResolver;
use rocket::http::{Header, Status};
use config::IDENTITY_CONFIG;
use tokio::sync::RwLock;
use anyhow::Result;
use xrpc_server::error::XRPCError;

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
async fn all_options() {
    /* Intentionally left empty */
}

#[derive(Debug)]
pub struct SharedIdResolver {
    pub id_resolver: RwLock<IdResolver>,
}

#[catch(default)]
async fn default_catcher(status: Status, _request: &Request<'_>) -> XRPCError {
    match status.code {
        400 => XRPCError::BadRequest,
        401 => XRPCError::Unauthorized,
        403 => XRPCError::Forbidden,
        404 => XRPCError::NotFound,
        413 => XRPCError::PayloadTooLarge,
        429 => XRPCError::TooManyRequests,
        501 => XRPCError::NotImplemented,
        500 => XRPCError::InternalServerError,
        _ => XRPCError::InternalServerError
    }
}

struct CORS;

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
    let shield = Shield::default().enable(NoSniff::Enable);

    let id_resolver = SharedIdResolver {
        id_resolver: RwLock::new(IdResolver::new(IdentityResolverOpts {
            timeout: None,
            plc_url: Some(IDENTITY_CONFIG.plc_url.clone()),
            did_cache: Some(DidCache::new(None, None)),
            backup_nameservers: IDENTITY_CONFIG.handle_backup_name_servers.clone()
        })),
    };

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let rocket = rocket::build()
        .mount("/", routes![all_options])
        .mount("/", api::routes())
        .manage(id_resolver)
        .manage(client)
        .attach(shield)
        .attach(CORS)
        .register("/", catchers![default_catcher]);

    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = init().await?;

    rocket.launch().await?;

    Ok(())
}

mod auth_verifier;
mod xrpc_server;
mod database;
mod helpers;
mod config;
mod api;
pub use appview_schema::schema;
