use crate::api::app::bsky::util::get_did_doc;
use crate::auth_verifier::AccessStandard;
use crate::config::BSKY_APP_VIEW_CONFIG;
use crate::{context, APP_USER_AGENT};
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use rsky_pds::repo::types::Ids;
use rsky_pds::common::get_notif_endpoint;
use rsky_pds::SharedIdResolver;
use anyhow::{anyhow, bail, Result};
use atrium_api::app::bsky::notification::register_push::{
    Input as AppBskyNotificationRegisterPushInput, InputData as AppBskyNotificationRegisterPushData,
};
use atrium_api::client::AtpServiceClient;
use atrium_api::types::string::Did;
use atrium_ipld::ipld::Ipld as AtriumIpld;
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::app::bsky::notification::RegisterPushInput;

pub async fn inner_register_push(
    body: Json<RegisterPushInput>,
    auth: AccessStandard,
    app_view_url: String,
    id_resolver: &State<SharedIdResolver>,
) -> Result<()> {
    let RegisterPushInput {
        service_did,
        token,
        platform,
        app_id,
    } = body.into_inner();
    let did: String = match auth.access.credentials {
        None => "".to_string(),
        Some(credentials) => credentials.did.unwrap_or("".to_string()),
    };
    let nsid = Ids::AppBskyFeedGetFeedGenerator.as_str().to_string();
    let auth_headers = context::service_auth_headers(&did, &service_did, &nsid).await?;

    let client = ReqwestClientBuilder::new(app_view_url)
        .client(
            reqwest::ClientBuilder::new()
                .user_agent(APP_USER_AGENT)
                .timeout(std::time::Duration::from_millis(1000))
                .default_headers(auth_headers.clone())
                .build()
                .unwrap(),
        )
        .build();
    let agent = AtpServiceClient::new(client);

    if let Some(bsky_app_view) = &*BSKY_APP_VIEW_CONFIG {
        if bsky_app_view.did == service_did {
            let _ = agent
                .service
                .app
                .bsky
                .notification
                .register_push(AppBskyNotificationRegisterPushInput {
                    data: AppBskyNotificationRegisterPushData {
                        app_id,
                        platform,
                        service_did: Did::new(service_did).map_err(|e| anyhow!(e))?,
                        token,
                    },
                    extra_data: AtriumIpld::Null,
                })
                .await?;
            return Ok(());
        }
    }
    let notif_endpoint = get_endpoint(id_resolver, service_did.clone()).await?;
    let client = ReqwestClientBuilder::new(notif_endpoint)
        .client(
            reqwest::ClientBuilder::new()
                .user_agent(APP_USER_AGENT)
                .timeout(std::time::Duration::from_millis(1000))
                .default_headers(auth_headers)
                .build()
                .unwrap(),
        )
        .build();
    let agent = AtpServiceClient::new(client);
    let _ = agent
        .service
        .app
        .bsky
        .notification
        .register_push(AppBskyNotificationRegisterPushInput {
            data: AppBskyNotificationRegisterPushData {
                app_id,
                platform,
                service_did: Did::new(service_did).map_err(|e| anyhow!(e))?,
                token,
            },
            extra_data: AtriumIpld::Null,
        })
        .await?;
    Ok(())
}

#[rocket::post(
    "/xrpc/app.bsky.notification.registerPush",
    format = "json",
    data = "<body>"
)]
pub async fn register_push(
    body: Json<RegisterPushInput>,
    auth: AccessStandard,
    id_resolver: &State<SharedIdResolver>,
) -> Result<(), status::Custom<Json<ErrorMessageResponse>>> {
    if !vec!["ios", "android", "web"].contains(&body.platform.as_str()) {
        let bad_request = ErrorMessageResponse {
            code: Some(ErrorCode::BadRequest),
            message: Some("invalid platform".to_string()),
        };
        return Err(status::Custom(Status::BadRequest, Json(bad_request)));
    }
    match &*BSKY_APP_VIEW_CONFIG {
        None => {
            let not_found = ErrorMessageResponse {
                code: Some(ErrorCode::NotFound),
                message: Some("not found".to_string()),
            };
            return Err(status::Custom(Status::NotFound, Json(not_found)));
        }
        Some(bsky_app_view) => {
            match inner_register_push(body, auth, bsky_app_view.url.clone(), id_resolver).await
            {
                Ok(_) => Ok(()),
                Err(error) => {
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
    }
}

pub async fn get_endpoint(
    id_resolver: &State<SharedIdResolver>,
    service_did: String,
) -> Result<String> {
    let doc = get_did_doc(id_resolver, &service_did).await?;
    match get_notif_endpoint(doc) {
        None => bail!("invalid notification service details in did document: {service_did}"),
        Some(notif_endpoint) => Ok(notif_endpoint),
    }
}