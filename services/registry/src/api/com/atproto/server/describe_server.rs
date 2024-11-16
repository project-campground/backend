use rsky_pds::models::ErrorMessageResponse;
use rocket::response::status;
use rocket::serde::json::Json;
use rsky_lexicon::com::atproto::server::{
    DescribeServerOutput, DescribeServerRefContact, DescribeServerRefLinks,
};

use crate::config::{CORE_CONFIG, IDENTITY_CONFIG, SERVICE_CONFIG};

#[rocket::get("/xrpc/com.atproto.server.describeServer")]
pub async fn describe_server(
) -> Result<Json<DescribeServerOutput>, status::Custom<Json<ErrorMessageResponse>>> {
    let available_user_domains = IDENTITY_CONFIG.service_handle_domains.clone();
    let privacy_policy = CORE_CONFIG.privacy_policy_url.clone();
    let terms_of_service = CORE_CONFIG.terms_of_service_url.clone();
    let contact_email_address = CORE_CONFIG.contact_email_address.clone();

    Ok(Json(DescribeServerOutput {
        did: SERVICE_CONFIG.did.clone(),
        available_user_domains,
        invite_code_required: None,
        phone_verification_required: None,
        links: DescribeServerRefLinks {
            privacy_policy,
            terms_of_service,
        },
        contact: DescribeServerRefContact {
            email: contact_email_address,
        },
    }))
}