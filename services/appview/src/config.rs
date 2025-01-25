#![allow(dead_code, unused_imports)]
use std::sync::LazyLock;
use rocket::{figment::Figment, serde::Deserialize};
use atrium_oauth_client::OAuthClientMetadata;
use lazy_static::lazy_static;
use rocket::Config;

lazy_static! {
    static ref CONFIG: Figment = Config::figment();
}

// Using statics for configs so that they can be accessed outside of a rocket context.
pub static DATABASE_CONFIG: LazyLock<DatabaseConfig> = LazyLock::new(|| CONFIG.extract_inner("database").expect("Failed to load database configuration"));
pub static IDENTITY_CONFIG: LazyLock<IdentityConfig> = LazyLock::new(|| CONFIG.extract_inner("identity").expect("Failed to load identity configuration"));
pub static CORE_CONFIG: LazyLock<CoreConfig> = LazyLock::new(|| CONFIG.extract_inner("core").expect("Failed to load core configuration"));
pub static EMAIL_CONFIG: LazyLock<MailConfig> = LazyLock::new(|| CONFIG.extract_inner("email").expect("Failed to load email configuration"));

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct CoreConfig {
    pub hostname: Option<String>,
    pub frontend: String,
    pub did: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub terms_of_service_url: Option<String>,
    pub contact_email_address: Option<String>,
    pub dev_mode: Option<bool>,
}

impl CoreConfig {
    pub fn public_url(&self) -> String {
        let hostname = self.hostname();
        if hostname == "localhost" {
            let port = CONFIG.extract_inner::<u16>("port").unwrap_or(8000);
            format!("http://localhost:{}", port)
        } else {
            format!("https://{}", hostname)
        }
    }

    pub fn hostname(&self) -> String {
        self.hostname.clone().unwrap_or("localhost".to_string())
    }

    pub fn did(&self) -> String {
        self.did.clone().unwrap_or(format!("did:web:{}", self.hostname()))
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode.unwrap_or(cfg!(debug_assertions))
    }

    pub fn oauth_metadata(&self) -> OAuthClientMetadata {
        OAuthClientMetadata {
            client_id: format!("{}/oauth/client-metadata.json", self.public_url()),
            client_uri: Some(self.public_url()),
            redirect_uris: vec![format!("{}/oauth/callback", self.frontend)],
            dpop_bound_access_tokens: Some(true),
            grant_types: Some(vec!["authorization_code".to_string(), "refresh_token".to_string()]),
            scope: Some("atproto transition:generic".to_string()),
            token_endpoint_auth_method: None,
            token_endpoint_auth_signing_alg: None,
            jwks: None,
            jwks_uri: None
        }
    }
}
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "provider")]
pub enum MailConfig {
    SMTP {
        host: String,
        username: String,
        password: String,
        from_address: String
    },
    Mailgun {
        api_key: String,
        domain: String,
        from_name: String,
        from_address: String
    }
}
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct IdentityConfig {
    pub plc_url: String,
    pub resolver_timeout: u64,
    pub cache_state_ttl: u64,
    pub cache_max_ttl: u64,
    pub recovery_did_key: Option<String>,
    pub service_handle_domains: Vec<String>,
    pub handle_backup_name_servers: Option<Vec<String>>,
    pub enable_did_doc_with_session: bool,
    pub reserved_handles_path: Option<String>,
    pub use_default_reserved_handles: Option<bool>,
    pub filter_explicit_handles: Option<bool>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}