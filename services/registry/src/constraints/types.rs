use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Deserialize, Serialize)]
pub struct Constraints {
    pub avatar: Constraint,
    pub banner: Constraint,
    pub embed: Constraint,
}