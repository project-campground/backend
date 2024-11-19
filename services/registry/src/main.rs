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
use account_manager::AccountManager;
use api::bsky_api_forwarder;
use config::{BSKY_APP_VIEW_CONFIG, S3_CONFIG};
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
use crate::read_after_write::viewer::{LocalViewerCreator, LocalViewer, LocalViewerCreatorParams};
use crate::sequencer::Sequencer;
use atrium_api::client::AtpServiceClient;
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use crate::config::{IDENTITY_CONFIG, CORE_CONFIG};
use event_emitter_rs::EventEmitter;
use lazy_static::lazy_static;

#[macro_use] extern crate rocket;

mod read_after_write;
mod account_manager;
mod auth_verifier;
mod pipethrough;
mod well_known;
mod repository;
mod sequencer;
mod database;
mod context;
mod mailer;
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

#[allow(missing_debug_implementations)]
pub struct SharedLocalViewer {
    pub local_viewer: RwLock<LocalViewerCreator>,
}

#[allow(missing_debug_implementations)]
pub struct SharedATPAgent {
    pub app_view_agent: Option<RwLock<AtpServiceClient<ReqwestClient>>>,
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
async fn default_catcher(status: Status, _request: &Request<'_>) -> Json<rsky_pds::models::ErrorMessageResponse> {
    let internal_error = rsky_pds::models::ErrorMessageResponse {
        code: Some(
            match status.code {
                400 => rsky_pds::models::ErrorCode::BadRequest,
                401 => rsky_pds::models::ErrorCode::Unauthorized,
                403 => rsky_pds::models::ErrorCode::Forbidden,
                404 => rsky_pds::models::ErrorCode::NotFound,
                409 => rsky_pds::models::ErrorCode::Conflict,
                500 => rsky_pds::models::ErrorCode::InternalServerError,
                503 => rsky_pds::models::ErrorCode::ServiceUnavailable,
                _ => rsky_pds::models::ErrorCode::InternalServerError
            }
        ),
        message: match status.code {
            400 => Some(status.reason().unwrap_or("Bad request.").to_string()),
            401 => Some(status.reason().unwrap_or("Unauthorized.").to_string()),
            403 => Some(status.reason().unwrap_or("Forbidden.").to_string()),
            404 => Some(status.reason().unwrap_or("Not found.").to_string()),
            409 => Some(status.reason().unwrap_or("Conflict.").to_string()),
            503 => Some(status.reason().unwrap_or("Service unavailable.").to_string()),
            500 => Some("Internal error.".to_string()),
            _ => Some("Internal error.".to_string())
        },
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

    let aws_sdk_config = S3_CONFIG.to_sdk_config().await;

    let id_resolver = SharedIdResolver {
        id_resolver: RwLock::new(IdResolver::new(IdentityResolverOpts {
            timeout: None,
            plc_url: Some(IDENTITY_CONFIG.plc_url.clone()),
            did_cache: Some(DidCache::new(None, None)),
            backup_nameservers: IDENTITY_CONFIG.handle_backup_name_servers.clone()
        })),
    };

    let app_view_agent = match &*BSKY_APP_VIEW_CONFIG {
        None => SharedATPAgent {
            app_view_agent: None,
        },
        Some(ref bsky_app_view) => {
            let client = ReqwestClientBuilder::new(bsky_app_view.url.clone())
                .client(
                    reqwest::ClientBuilder::new()
                        .user_agent(APP_USER_AGENT)
                        .timeout(std::time::Duration::from_millis(1000))
                        .build()
                        .unwrap(),
                )
                .build();
            SharedATPAgent {
                app_view_agent: Some(RwLock::new(AtpServiceClient::new(client))),
            }
        }
    };
    let local_viewer = SharedLocalViewer {
        local_viewer: RwLock::new(LocalViewer::creator(LocalViewerCreatorParams {
            account_manager: AccountManager {},
            pds_hostname: CORE_CONFIG.hostname().clone(),
            appview_agent: match &*BSKY_APP_VIEW_CONFIG {
                None => None,
                Some(ref bsky_app_view) => Some(bsky_app_view.url.clone()),
            },
            appview_did: match &*BSKY_APP_VIEW_CONFIG {
                None => None,
                Some(ref bsky_app_view) => Some(bsky_app_view.did.clone()),
            },
            appview_cdn_url_pattern: match &*BSKY_APP_VIEW_CONFIG {
                None => None,
                Some(ref bsky_app_view) => bsky_app_view.cdn_url_pattern.clone(),
            },
        })),
    };

    let shield = Shield::default().enable(NoSniff::Enable);

    let rocket = rocket::build()
        .mount("/", routes![
            api::com::atproto::identity::resolve_handle::resolve_handle,
            api::com::atproto::identity::update_handle::update_handle,
            api::com::atproto::server::create_account::create_account,
            api::com::atproto::server::deactivate_account::deactivate_account,
            api::com::atproto::server::activate_account::activate_account,
            api::com::atproto::server::delete_account::delete_account,
            api::com::atproto::server::create_session::create_session,
            api::com::atproto::server::delete_session::delete_session,
            api::com::atproto::server::get_session::get_session,
            api::com::atproto::server::refresh_session::refresh_session,
            api::com::atproto::server::get_service_auth::get_service_auth,
            api::com::atproto::server::describe_server::describe_server,
            api::com::atproto::server::reset_password::reset_password,
            api::com::atproto::server::update_email::update_email,
            api::com::atproto::server::get_account_invite_codes::get_account_invite_codes,
            api::com::atproto::server::reserve_signing_key::reserve_signing_key,
            api::com::atproto::server::check_account_status::check_account_status,
            api::com::atproto::server::request_account_delete::request_account_delete,
            api::com::atproto::server::request_email_confirmation::request_email_confirmation,
            api::com::atproto::server::request_password_reset::request_password_reset,
            api::com::atproto::server::request_email_update::request_email_update,
            api::com::atproto::server::confirm_email::confirm_email,
            api::com::atproto::server::create_app_password::create_app_password,
            api::com::atproto::server::list_app_passwords::list_app_passwords,
            api::com::atproto::server::revoke_app_password::revoke_app_password,
            api::com::atproto::repo::apply_writes::apply_writes,
            api::com::atproto::repo::create_record::create_record,
            api::com::atproto::repo::delete_record::delete_record,
            api::com::atproto::repo::describe_repo::describe_repo,
            api::com::atproto::repo::get_record::get_record,
            api::com::atproto::repo::import_repo::import_repo,
            api::com::atproto::repo::list_missing_blobs::list_missing_blobs,
            api::com::atproto::repo::list_records::list_records,
            api::com::atproto::repo::put_record::put_record,
            api::com::atproto::repo::upload_blob::upload_blob,
            api::com::atproto::sync::get_blob::get_blob,
            api::com::atproto::sync::get_blocks::get_blocks,
            api::com::atproto::sync::get_latest_commit::get_latest_commit,
            api::com::atproto::sync::get_record::get_record,
            api::com::atproto::sync::get_repo_status::get_repo_status,
            api::com::atproto::sync::get_repo::get_repo,
            api::com::atproto::sync::list_blobs::list_blobs,
            api::com::atproto::sync::list_repos::list_repos,
            api::com::atproto::sync::subscribe_repos::subscribe_repos,
            api::com::atproto::admin::delete_account::delete_account,
            api::com::atproto::admin::get_account_info::get_account_info,
            api::com::atproto::admin::get_subject_status::get_subject_status,
            api::com::atproto::admin::send_email::send_email,
            api::com::atproto::admin::update_account_email::update_account_email,
            api::com::atproto::admin::update_account_handle::update_account_handle,
            api::com::atproto::admin::update_account_password::update_account_password,
            api::com::atproto::admin::update_subject_status::update_subject_status,
            api::app::bsky::actor::get_preferences::get_preferences,
            api::app::bsky::actor::get_profile::get_profile,
            api::app::bsky::actor::get_profiles::get_profiles,
            api::app::bsky::actor::put_preferences::put_preferences,
            api::app::bsky::feed::get_actor_likes::get_actor_likes,
            api::app::bsky::feed::get_author_feed::get_author_feed,
            api::app::bsky::feed::get_feed::get_feed,
            api::app::bsky::feed::get_post_thread::get_post_thread,
            api::app::bsky::feed::get_timeline::get_timeline,
            api::app::bsky::notification::register_push::register_push,
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
        .manage(local_viewer)
        .manage(aws_sdk_config)
        .manage(id_resolver)
        .manage(app_view_agent);

    Ok(rocket)
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = init().await?;

    rocket.launch().await?;

    Ok(())
}