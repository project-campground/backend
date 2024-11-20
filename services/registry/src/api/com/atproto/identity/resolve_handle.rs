use crate::account_manager::helpers::account::ActorAccount;
use crate::account_manager::AccountManager;
use crate::api::com::atproto::server::normalize_and_ensure_valid_handle;
use crate::{SharedIdResolver, APP_USER_AGENT};
use crate::config::{BSKY_APP_VIEW_CONFIG, IDENTITY_CONFIG};
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use anyhow::{bail, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::identity::ResolveHandleOutput;

async fn try_resolve_from_app_view(handle: &String) -> Result<Option<String>> {
    match BSKY_APP_VIEW_CONFIG.as_ref() {
        None => Ok(None),
        Some(config) => {
            let client = reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()?;
            let params = Some(vec![("handle", handle)]);
            let res = client
                .get(format!(
                    "{0}/xrpc/com.atproto.identity.resolveHandle",
                    config.url
                ))
                .header("Connection", "Keep-Alive")
                .header("Keep-Alive", "timeout=5, max=1000")
                .query(&params)
                .send()
                .await;
            match res {
                Err(_) => Ok(None),
                Ok(res) => match res.json::<ResolveHandleOutput>().await {
                    Err(_) => Ok(None),
                    Ok(data) => Ok(Some(data.did)),
                },
            }
        }
    }
}

async fn inner_resolve_handle(
    handle: String,
    id_resolver: &State<SharedIdResolver>,
) -> Result<ResolveHandleOutput> {
    let handle = normalize_and_ensure_valid_handle(&handle)?;
    let mut did: Option<String> = None;
    let user: Option<ActorAccount> = AccountManager::get_account(&handle, None).await?;

    match user {
        Some(user) => did = Some(user.did),
        None => {
            let supported_handle = IDENTITY_CONFIG.service_handle_domains
                .iter()
                .find(|host| handle.ends_with(host.as_str()) || handle == host[1..])
                .is_some();
            // this should be in our DB & we couldn't find it, so fail
            if supported_handle {
                bail!("unable to resolve handle");
            }
        }
    }

    // this is not someone on our server, but we help with resolving anyway
    // @TODO: Weird error about Tokio received when this fails that leads to panic
    if did.is_none() && BSKY_APP_VIEW_CONFIG.is_some() {
        did = try_resolve_from_app_view(&handle).await?;
    }

    if did.is_none() {
        let mut lock = id_resolver.id_resolver.write().await;
        did = lock.handle.resolve(&handle).await?;
    }

    match did {
        None => bail!("unable to resolve handle"),
        Some(did) => Ok(ResolveHandleOutput { did }),
    }
}

#[rocket::get("/xrpc/com.atproto.identity.resolveHandle?<handle>")]
pub async fn resolve_handle(
    handle: String,
    id_resolver: &State<SharedIdResolver>,
) -> Result<Json<ResolveHandleOutput>, status::Custom<Json<ErrorMessageResponse>>> {
    match inner_resolve_handle(handle, id_resolver).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            eprintln!("@LOG: ERROR: {error}");
            let internal_error = ErrorMessageResponse {
                code: Some(ErrorCode::InternalServerError),
                message: Some(error.to_string()),
            };
            return Err(status::Custom(
                Status::InternalServerError,
                Json(internal_error),
            ));
        }
    }
}