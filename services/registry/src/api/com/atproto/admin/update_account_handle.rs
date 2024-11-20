use crate::account_manager::helpers::account::AvailabilityFlags;
use crate::account_manager::AccountManager;
use crate::api::com::atproto::server::{get_keys_from_private_key_str, normalize_and_validate_handle};
use crate::auth_verifier::AdminToken;
use crate::SharedSequencer;
use crate::config::{IDENTITY_CONFIG, SECRET_CONFIG};
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use rsky_pds::plc;
use anyhow::{bail, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::admin::UpdateAccountHandleInput;

async fn inner_update_account_handle(
    body: Json<UpdateAccountHandleInput>,
    sequencer: &State<SharedSequencer>,
) -> Result<()> {
    let UpdateAccountHandleInput { did, handle } = body.into_inner();
    let account = AccountManager::get_account(
        &normalize_and_validate_handle(&handle)?,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: Some(true),
        }),
    )
    .await?;

    match account {
        Some(account) if account.did != did => bail!("Handle already taken: {handle}"),
        Some(_) => (),
        None => {
            let plc_url = IDENTITY_CONFIG.plc_url.clone();
            let plc_client = plc::Client::new(plc_url);
            let private_key = SECRET_CONFIG.pds_rotation_key.clone();
            let (signing_key, _) = get_keys_from_private_key_str(private_key)?;
            plc_client
                .update_handle(&did, &signing_key, &handle)
                .await?;
            AccountManager::update_handle(&did, &handle).await?;
        }
    }
    let mut lock = sequencer.sequencer.write().await;
    lock.sequence_identity_evt(did.clone(), Some(handle.clone()))
        .await?;
    lock.sequence_handle_update(did.clone(), handle.clone())
        .await?;
    Ok(())
}

#[rocket::post(
    "/xrpc/com.atproto.admin.updateAccountHandle",
    format = "json",
    data = "<body>"
)]
pub async fn update_account_handle(
    body: Json<UpdateAccountHandleInput>,
    sequencer: &State<SharedSequencer>,
    _auth: AdminToken,
) -> Result<(), status::Custom<Json<ErrorMessageResponse>>> {
    match inner_update_account_handle(body, sequencer).await {
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