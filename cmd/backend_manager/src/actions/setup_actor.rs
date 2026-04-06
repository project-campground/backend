use std::io::{self, Stdin, Write};

use appview_schema::{models::appview::{Actor, Profile}, schema::appview};
use chrono::Utc;
use diesel::RunQueryDsl;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, from_value};

use crate::{HTTP_CLIENT, database::establish_connection};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequestBody {
    email: String,
    password: String,
    handle: String,
    invite_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponseBody {
    did: String,
    access_jwt: String,
    // refresh_jwt: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PdsRecordBody<T> {
    repo: String,
    collection: String,
    rkey: String,
    record: PdsRecord<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PdsRecord<T> {
    #[serde(rename = "$type")]
    r#type: String,
    #[serde(flatten)]
    body: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HomeServerRecord {
    did: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRecord {
    display_name: String,
    tagline: Option<String>,
    location: Option<String>,
    description: Option<String>,
    social_connections: Vec<String>,
    labels: Vec<String>,
    created_at: String,
    avatar: Option<String>,
}

pub async fn setup_actor(client: &HTTP_CLIENT, database_url: &str, pds_port: u16) -> Result<(), String> {
    let pds_localhost_xrpc = format!("http://localhost:{}/xrpc", pds_port);

    // Required for closed PDS
    let invite_resp = client
        .post(format!("{}/com.atproto.server.createInviteCode", &pds_localhost_xrpc))
        .header("Content-Type", "application/json")
        .basic_auth("admin", Some("admin-pass"))
        .json(&InviteRequestBody {
            use_count: 1
        })
        .send()
        .await;

    let invite_code = deserialize_and_handle_response::<InviteResponseBody>(invite_resp, "Invite")
        .await?
        .code;
    println!("Invite code: {:?}", invite_code);

    // Just general creation from invite
    let mut stdin = io::stdin();

    let mut email = String::new();
    read_stdin(&mut stdin, &mut email, "email")?;
    let mut handle = String::new();
    read_stdin(&mut stdin, &mut handle, "handle (e.g. john.test with .test as preferred top-level domain for dev-env purposes)")?;
    let mut password = String::new();
    read_stdin(&mut stdin, &mut password, "password")?;
    
    let account_creation = client
        .post(format!("{}/com.atproto.server.createAccount", pds_localhost_xrpc))
        .header("Content-Type", "application/json")
        .json(&CreateAccountRequestBody {
            email,
            handle: handle.clone(),
            password,
            invite_code
        })
        .send()
        .await;

    let pds_account = deserialize_and_handle_response::<CreateAccountResponseBody>(account_creation, "Account")
        .await?;
    
    println!("PDS Account: {:?}", pds_account);
    let CreateAccountResponseBody { access_jwt, did } = &pds_account;

    let mut home_server_port = String::new();
    read_stdin(&mut stdin, &mut home_server_port, "back-end localhost port (so home server could be set as the appview, likely 3984 from Rocket.toml in /services/appview)")?;
    let home_server_port: u16 = home_server_port
        .parse()
        .map_err(|x| format!("Error while parsing the port: {}", x))?;

    let home_server_did = &format!("did:web:localhost%3A{}", home_server_port);
    put_record_in_pds(client, &pds_localhost_xrpc, "gg.campground.homeServer", did, access_jwt, HomeServerRecord {
        did: home_server_did.clone(),
    }).await?;
    
    let mut display_name = String::new();
    read_stdin(&mut stdin, &mut display_name, "display name for the profile")?;
    
    let mut tagline = String::new();
    read_stdin(&mut stdin, &mut tagline, "profile tagline (can be whatever)")?;
    
    let mut location = String::new();
    read_stdin(&mut stdin, &mut location, "profile location (can be whatever)")?;
    
    let mut description = String::new();
    read_stdin(&mut stdin, &mut description, "profile description (can be whatever)")?;

    let account_created_at = &Utc::now()
        .naive_utc()
        .format("%FT%H:%M:%S%.3fZ")
        .to_string();

    put_record_in_pds(client, &pds_localhost_xrpc, "gg.campground.actor.profile", did, access_jwt, ProfileRecord {
        display_name: display_name.clone(),
        tagline: Some(tagline.clone()),
        location: Some(location.clone()),
        description: Some(description.clone()),
        avatar: None,
        social_connections: vec![],
        labels: vec![],
        created_at: account_created_at.clone(),
    }).await?;

    println!("Inserted profile record. Inserting into DB");

    let mut conn = establish_connection(database_url).unwrap();

    diesel::insert_into(
        appview::actor::table
    )
        .values(
            Actor {
                did: did.clone(),
                handle: Some(format!("at://{}", &handle)),
                home_server: home_server_did.clone(),
                indexed_at: account_created_at.clone(),
                campsites: vec![],
            }
        )
        .execute(&mut conn)
        .map_err(|x| format!("Error while inserting actor into DB: {}", x))?;

    println!("Actor inserted");

    diesel::insert_into(
        appview::profile::table
    )
        .values(
            Profile {
                uri: format!("at://{}/gg.campground.actor.profile/self", did),
                creator: did.clone(),
                cid: "".to_string(),
                avatar_cid: None,
                banner_cid: None,
                display_name: Some(display_name.clone()),
                description: Some(description.clone()),
                location: Some(location.clone()),
                tagline: Some(tagline.clone()),
                indexed_at: account_created_at.clone(),
                created_at: Some(account_created_at.clone()),
                first_seen: account_created_at.clone(),
            }
        )
        .execute(&mut conn)
        .map_err(|x| format!("Error while inserting profile into DB: {}", x))?;

    println!("Profile inserted. Actor done creating. You should be able to login with the given password and username.");

    Ok(())
}

async fn put_record_in_pds<T: Serialize>(client: &HTTP_CLIENT, pds_localhost_xrpc: &str, record: &str, did: &String, access_jwt: &str, body: T) -> Result<Response, String> {
    client
        .post(format!("{}/com.atproto.repo.putRecord", pds_localhost_xrpc))
        .header("Content-Type", "application/json")
        .bearer_auth(access_jwt)
        .json(&PdsRecordBody::<T> {
            repo: did.clone(),
            collection: record.to_string(),
            rkey: "self".to_string(),
            record: PdsRecord {
                r#type: record.to_string(),
                body,
            },
        })
        .send()
        .await
        .map_err(|x| format!("Error while setting {} record: {}", record, x))?
        .error_for_status()
        .map_err(|x| format!("Error from PDS while setting {} record: {}", record, x))
}

fn read_stdin(stdin: &mut Stdin, input_value: &mut String, prompt: &str) -> Result<(), String> {
    print!("Enter {}: ", prompt);

    io::stdout()
        .flush()
        .map_err(|x| format!("STDOUT error: {:?}", x))?;

    stdin
        .read_line(input_value)
        .map_err(|x| format!("STDIN error: {:?}", x))?;

    // To remove trailing \n
    *input_value = input_value.split('\n').next().unwrap().to_string();

    Ok(())
}

async fn deserialize_and_handle_response<T: DeserializeOwned>(response_result: Result<Response, Error>, title: &str) -> Result<T, String> {
    let response_result = response_result
        .map_err(|x| format!("{} error: {:?}", title, x))?;
    let status = response_result.status().as_u16();

    let result_json = response_result.json::<Value>()
        .await
        .map_err(|x| format!("{} JSON error: {:?}", title, x))?;

    if status >= 400 {
        return Err(format!("{} response error: {:?}", title, result_json));
    }

    Ok(from_value::<T>(result_json)
        .map_err(|x| format!("Error while deserializing invite JSON: {:?}", x))?)
}