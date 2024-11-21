/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::helpers::account::AvailabilityFlags;
use crate::account_manager::AccountManager;
use crate::api::com::atproto::server::{get_keys_from_private_key_str, normalize_and_validate_handle};
use crate::auth_verifier::AccessStandardCheckTakedown;
use crate::config::{IDENTITY_CONFIG, SECRET_CONFIG};
use crate::SharedSequencer;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use crate::plc;
use anyhow::{bail, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::identity::UpdateHandleInput;

async fn inner_update_handle(
    body: Json<UpdateHandleInput>,
    sequencer: &State<SharedSequencer>,
    auth: AccessStandardCheckTakedown,
) -> Result<()> {
    let UpdateHandleInput { handle } = body.into_inner();
    let requester = auth.access.credentials.unwrap().did.unwrap();

    let handle = normalize_and_validate_handle(&handle)?;
    let account = AccountManager::get_account(
        &handle,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: None,
        }),
    )
    .await?;

    match account {
        Some(account) if account.did != requester => bail!("Handle already taken: {handle}"),
        Some(_) => (),
        None => {
            let plc_url = IDENTITY_CONFIG.plc_url.clone();
            let plc_client = plc::Client::new(plc_url);
            let private_key = SECRET_CONFIG.pds_rotation_key.clone();
            let (signing_key, _) = get_keys_from_private_key_str(private_key)?;
            plc_client
                .update_handle(&requester, &signing_key, &handle)
                .await?;
            AccountManager::update_handle(&requester, &handle).await?;
        }
    }
    let mut lock = sequencer.sequencer.write().await;
    match lock
        .sequence_identity_evt(requester.clone(), Some(handle.clone()))
        .await
    {
        Ok(_) => (),
        Err(error) => eprintln!(
            "Error: {}; DID: {}; Handle: {}",
            error.to_string(),
            &requester,
            &handle
        ),
    };
    match lock
        .sequence_handle_update(requester.clone(), handle.clone())
        .await
    {
        Ok(_) => (),
        Err(error) => eprintln!(
            "Error: {}; DID: {}; Handle: {}",
            error.to_string(),
            &requester,
            &handle
        ),
    };
    Ok(())
}

#[rocket::post(
    "/xrpc/com.atproto.identity.updateHandle",
    format = "json",
    data = "<body>"
)]
pub async fn update_handle(
    body: Json<UpdateHandleInput>,
    sequencer: &State<SharedSequencer>,
    auth: AccessStandardCheckTakedown,
) -> Result<(), status::Custom<Json<ErrorMessageResponse>>> {
    match inner_update_handle(body, sequencer, auth).await {
        Ok(_) => Ok(()),
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