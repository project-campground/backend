/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * Modified to work with our own DB
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */
use crate::account_manager::AccountManager;
use crate::auth_verifier::AccessFull;
use rsky_pds::models::{ErrorCode, ErrorMessageResponse};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rsky_lexicon::com::atproto::server::RevokeAppPasswordInput;

#[rocket::post(
    "/xrpc/com.atproto.server.revokeAppPassword",
    format = "json",
    data = "<body>"
)]
pub async fn revoke_app_password(
    body: Json<RevokeAppPasswordInput>,
    auth: AccessFull,
) -> Result<(), status::Custom<Json<ErrorMessageResponse>>> {
    let RevokeAppPasswordInput { name } = body.into_inner();
    let requester = auth.access.credentials.unwrap().did.unwrap();

    match AccountManager::revoke_app_password(requester, name).await {
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