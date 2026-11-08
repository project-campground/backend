use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleMotion {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "linear")]
    Linear,
    #[serde(alias = "wave")]
    Wave,
    #[serde(alias = "radial")]
    Radial,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleViewBasic {
    pub id: Uuid,
    pub campsite_id: String,

    pub name: String,
    pub raised: bool,
    pub pingable: bool,
    pub flags: i32,

    pub permissions: u32,
    pub position: i32,

    pub colors: Vec<u32>,
    pub motion: RoleMotion,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRolesOutput {
    pub roles: Vec<RoleViewBasic>,
}
