/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::helpers::account::AvailabilityFlags;
use crate::account_manager::AccountManager;
use crate::auth_verifier::Moderator;
use crate::INVALID_HANDLE;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use anyhow::{bail, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rsky_lexicon::com::atproto::admin::AccountView;

async fn inner_get_account_info(did: String) -> Result<AccountView> {
    let account = AccountManager::get_account(
        &did,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: Some(true)
        })
    ).await?;
    if let Some(account) = account {
        Ok(AccountView {
            did: account.did,
            handle: account.handle.unwrap_or(INVALID_HANDLE.to_string()),
            email: account.email,
            indexed_at: account.created_at,
            email_confirmed_at: account.email_confirmed_at,
            invited_by: None,
            invites: None,
            invites_disabled: None,
            related_records: None,
            invite_note: None,
        })
    } else {
        bail!("Account not found")
    }
}

#[rocket::get("/xrpc/com.atproto.admin.getAccountInfo?<did>")]
pub async fn get_account_info(
    did: String,
    _auth: Moderator,
) -> Result<Json<AccountView>, status::Custom<Json<ErrorMessageResponse>>> {
    match inner_get_account_info(did).await {
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