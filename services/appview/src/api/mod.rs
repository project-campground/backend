use atrium_api::did_doc::{DidDocument, Service, VerificationMethod};
use rsky_crypto::utils::encode_did_key;

use crate::config::CORE_CONFIG;

macro_rules! merge_routes {
    ($($routes:expr),*) => {{
        let mut routes = Vec::new();
        $(
            routes.extend($routes);
        )*
        routes
    }}
}

#[get("/robots.txt")]
async fn robots() -> &'static str {
    "# Hello!\n\n# Crawling the public API is allowed. HTTP 429 (\"backoff\") status codes are used for rate-limiting.\nUser-agent: *\nAllow: /"
}

#[get("/")]
async fn index() -> &'static str {
    "This is an AT Protocol Application View (AppView) for the \"campground.gg\" application.\n\nMost API routes are under /xrpc/\n\nCode: https://github.com/Project-Campground/backend\nProtocol: https://atproto.com"
}

#[get("/oauth/client-metadata.json")]
async fn oauth_client_metadata() -> String {
    serde_json::to_string(&CORE_CONFIG.oauth_metadata()).unwrap()
}

#[get("/.well-known/did.json")]
async fn did() -> String {
    let doc = DidDocument {
        id: CORE_CONFIG.did().clone(),
        also_known_as: None,
        context: None,
        // context: Some(vec![
        //     "https://www.w3.org/ns/did/v1".to_string(),
        //     "https://w3id.org/security/multikey/v1".to_string(),
        //     "https://w3id.org/security/suites/secp256k1-2019/v1".to_string()
        // ]),
        verification_method: Some(vec![
            VerificationMethod {
                id: format!("{}#atproto", CORE_CONFIG.did()),
                r#type: "Multikey".to_string(),
                controller: CORE_CONFIG.did(),
                public_key_multibase: Some(
                    encode_did_key(&CORE_CONFIG.verification_keypair().public_key()).replace("did:key:", "")
                )
            }
        ]),
        service: Some(vec![
            Service {
                id: "#campground_appview".to_string(),
                r#type: "CampgroundAppview".to_string(),
                service_endpoint: CORE_CONFIG.public_url()
            }
        ])
    };

    serde_json::to_string(&doc).unwrap()
}

pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        gg::routes(),
        ws::routes(),
        rocket::routes![
            robots,
            index,
            oauth_client_metadata,
            did
        ]
    )
}

mod gg;
mod ws;
