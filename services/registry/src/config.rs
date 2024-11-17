#![allow(dead_code, unused_imports)]
use std::sync::LazyLock;

use aws_config::SdkConfig;
use rocket::{figment::Figment, serde::Deserialize};
use lazy_static::lazy_static;
use rocket::Config;

lazy_static! {
    static ref CONFIG: Figment = Config::figment();
}

// Using statics for configs so that they can be accessed outside of a rocket context.
pub static DATABASE_CONFIG: LazyLock<DatabaseConfig> = LazyLock::new(|| CONFIG.extract_inner("database").expect("Failed to load database configuration"));
pub static IDENTITY_CONFIG: LazyLock<IdentityConfig> = LazyLock::new(|| CONFIG.extract_inner("identity").expect("Failed to load identity configuration"));
pub static CORE_CONFIG: LazyLock<CoreConfig> = LazyLock::new(|| CONFIG.extract_inner("core").expect("Failed to load core configuration"));
pub static SECRET_CONFIG: LazyLock<SecretConfig> = LazyLock::new(|| CONFIG.extract_inner("secret").expect("Failed to load secret configuration"));
pub static EMAIL_CONFIG: LazyLock<MailConfig> = LazyLock::new(|| CONFIG.extract_inner("email").expect("Failed to load email configuration"));
pub static MODERATION_EMAIL_CONFIG: LazyLock<MailConfig> = LazyLock::new(|| CONFIG.extract_inner("mod_email").expect("Failed to load moderation email configuration"));
pub static S3_CONFIG: LazyLock<S3Config> = LazyLock::new(|| CONFIG.extract_inner("s3").expect("Failed to load AWS configuration"));

pub static SERVICE_CONFIG: LazyLock<ServiceConfig> = LazyLock::new(|| CONFIG.extract_inner("service").expect("Failed to load service configuration"));
pub static MOD_SERVICE_CONFIG: LazyLock<Option<ServiceConfig>> = LazyLock::new(|| CONFIG.extract_inner("mod_service").unwrap_or(None));
pub static ENTRYWAY_CONFIG: LazyLock<Option<ServiceConfig>> = LazyLock::new(|| CONFIG.extract_inner("entryway").unwrap_or(None));
pub static REPORT_SERVICE_CONFIG: LazyLock<Option<ServiceConfig>> = LazyLock::new(|| CONFIG.extract_inner("report_service").unwrap_or(None));
pub static BSKY_APP_VIEW_CONFIG: LazyLock<Option<ServiceConfig>> = LazyLock::new(|| CONFIG.extract_inner("bsky_app_view").unwrap_or(None));

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SecretConfig {
    pub pds_private_key: String,
    pub pds_rotation_key: String,
    pub repo_signing_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct CoreConfig {
    pub hostname: Option<String>,
    pub did: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub terms_of_service_url: Option<String>,
    pub blob_upload_limit: Option<usize>,
    pub contact_email_address: Option<String>,
    pub aws_endpoint: Option<String>,
    pub dev_mode: Option<bool>,
    pub crawlers: Vec<String>,
    pub admin_pass: String,
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

    pub fn blob_upload_limit(&self) -> usize {
        self.blob_upload_limit.unwrap_or(5 * 1024 * 1024) // 5 MB
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode.unwrap_or(cfg!(debug_assertions))
    }
}
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

impl S3Config {
    pub async fn to_sdk_config(&self) -> SdkConfig {
        aws_config::ConfigLoader::default()
            .endpoint_url(self.endpoint.to_owned())
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                self.access_key.to_owned(),
                self.secret_key.to_owned(),
                None,
                None,
                "custom",
            ))
            .region(aws_sdk_s3::config::Region::new(self.region.to_owned()))
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .load()
            .await
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
pub struct ServiceConfig {
    pub url: String,
    pub did: String,
    pub cdn_url_pattern: Option<String>, // for BksyAppViewConfig, otherwise None
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
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}