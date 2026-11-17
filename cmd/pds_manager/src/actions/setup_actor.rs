use reqwest::{Error, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, from_value};

use crate::HTTP_CLIENT;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InviteRequestBody {
    use_count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InviteResponseBody {
    code: String,
}

pub async fn setup_actor(client: &HTTP_CLIENT, pds_port: u16) -> Result<(), String> {
    let pds_localhost_xrpc = format!("http://localhost:{}/xrpc", pds_port);

    // Required for closed PDS
    let invite_resp = client
        .post(format!(
            "{}/com.atproto.server.createInviteCode",
            &pds_localhost_xrpc
        ))
        .header("Content-Type", "application/json")
        .basic_auth("admin", Some("admin-pass"))
        .json(&InviteRequestBody { use_count: 1 })
        .send()
        .await;

    let invite_code = deserialize_and_handle_response::<InviteResponseBody>(invite_resp, "Invite")
        .await?
        .code;
    println!("Invite code: {:?}", invite_code);

    Ok(())
}

async fn deserialize_and_handle_response<T: DeserializeOwned>(
    response_result: Result<Response, Error>,
    title: &str,
) -> Result<T, String> {
    let response_result = response_result.map_err(|x| format!("{} error: {:?}", title, x))?;
    let status = response_result.status().as_u16();

    let result_json = response_result
        .json::<Value>()
        .await
        .map_err(|x| format!("{} JSON error: {:?}", title, x))?;

    if status >= 400 {
        return Err(format!("{} response error: {:?}", title, result_json));
    }

    Ok(from_value::<T>(result_json)
        .map_err(|x| format!("Error while deserializing invite JSON: {:?}", x))?)
}
