use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use surrealdb::sql::Thing;

pub trait DBModel {
    fn id(&self) -> String;
    fn table(&self) -> String;
}

macro_rules! impl_db_model {
    ($($type:ident),*) => {
        $(
            impl DBModel for $type {
                fn id(&self) -> String {
                    self.id.id.to_string()
                }

                fn table(&self) -> String {
                    self.id.tb.to_string()
                }
            }
        )*
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: Thing,
    pub created_at: DateTime<Utc>,
    pub email: String,
    pub password: String,
    pub email_confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    pub id: Thing,
    pub handle: String,
    pub created_at: DateTime<Utc>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub delete_after: Option<DateTime<Utc>>,
    pub takedown_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppPassword {
    pub id: Thing,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub password: String,
    pub privileged: bool,
}

impl_db_model![Account, Actor, AppPassword];