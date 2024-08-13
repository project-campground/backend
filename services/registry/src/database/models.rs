use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use surrealdb::sql::Thing;

pub trait DBModel {
    fn id(&self) -> String;
    fn table(&self) -> String;
}

macro_rules! impl_db_model {
    ($($types:ident),*) => {
        $(
            impl DBModel for $types {
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
    id: Thing,
    created_at: DateTime<Utc>,
    email: String,
    password: String,
    email_confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    id: Thing,
    handle: String,
    created_at: DateTime<Utc>,
    deactivated_at: Option<DateTime<Utc>>,
    delete_after: Option<DateTime<Utc>>,
    takedown_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppPassword {
    id: Thing,
    name: String,
    created_at: DateTime<Utc>,
    password: String,
    privileged: bool,
}

impl_db_model![Account, Actor, AppPassword];