use rsky_pds::account_manager::helpers::auth::ServiceJwtParams;
use rsky_pds::xrpc_server::auth::create_service_auth_headers;
use crate::config::SECRET_CONFIG;
use anyhow::Result;
use reqwest::header::HeaderMap;
use secp256k1::SecretKey;

pub async fn service_auth_headers(did: &String, aud: &String, lxm: &String) -> Result<HeaderMap> {
    let private_key = &SECRET_CONFIG.repo_signing_key;
    let keypair = SecretKey::from_slice(&hex::decode(private_key.as_bytes()).unwrap()).unwrap();
    create_service_auth_headers(ServiceJwtParams {
        iss: did.clone(),
        aud: aud.clone(),
        exp: None,
        lxm: Some(lxm.clone()),
        jti: None,
        keypair,
    })
    .await
}