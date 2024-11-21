/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::helpers::account::AvailabilityFlags;
use crate::account_manager::AccountManager;
use crate::auth_verifier::AccessStandardIncludeChecks;
use crate::mailer;
use crate::mailer::IdentifierAndTokenParams;
use crate::database::models::EmailTokenPurpose;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use anyhow::{bail, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;

async fn inner_request_account_delete(auth: AccessStandardIncludeChecks) -> Result<()> {
    let did = auth.access.credentials.unwrap().did.unwrap();
    let account = AccountManager::get_account(
        &did,
        Some(AvailabilityFlags {
            include_deactivated: Some(true),
            include_taken_down: Some(true),
        }),
    )
    .await?;
    if let Some(account) = account {
        if let Some(email) = account.email {
            let token =
                AccountManager::create_email_token(&did, EmailTokenPurpose::DeleteAccount).await?;
            mailer::send_account_delete(email.clone(), IdentifierAndTokenParams { token, identifier: account.handle.as_ref().unwrap_or(&email).to_owned() }).await?;
            Ok(())
        } else {
            bail!("Account does not have an email address")
        }
    } else {
        bail!("Account not found")
    }
}

#[rocket::post("/xrpc/com.atproto.server.requestAccountDelete")]
pub async fn request_account_delete(
    auth: AccessStandardIncludeChecks,
) -> Result<(), status::Custom<Json<ErrorMessageResponse>>> {
    match inner_request_account_delete(auth).await {
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