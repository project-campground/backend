use crate::config::CORE_CONFIG;

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

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![robots, index, oauth_client_metadata]
}
