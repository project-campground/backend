use did_method_plc::Keypair;
use rocket::State;
use serde_json::json;

use crate::config;

#[get("/did.json")]
async fn did(
    auth_config: &State<config::AuthConfig>,
    #[allow(unused_variables)]
    service_config: &State<config::ServiceConfig>,
) -> String {
    let key = Keypair::from_private_key(&auth_config.secret_key);
    match key {
        #[allow(unused_variables)]
        Ok(key) => {
            let key_did = key.to_did_key().unwrap();
            json!({
                "@context": ["https://www.w3.org/ns/did/v1"],
                "id": service_config.did,
                "verificationMethod": [
                    {
                        "id": format!("{}#atproto_label", service_config.did),
                        "type": "Multikey",
                        "controller": service_config.did,
                        "publicKeyMultibase": key_did.replace("did:key", ""),
                    }
                ],
                "service": [
                    {
                        "id": "#atproto_labeler",
                        "type": "AtprotoLabeler",
                        "service_endpoint": service_config.public_url,
                    }
                ]
            }).to_string()
        }
        Err(e) => {
            panic!("Failed to create DID: {}", e);
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![did]
}