#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsDictionary {
    pub general: i64,
    pub content: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsStateDictionary {
    pub allowed: PermissionsDictionary,
    pub denied: PermissionsDictionary,
}
