use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: DateTime<Utc>,

    pub handle: String,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar_url: String,
}