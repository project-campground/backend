#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsDictionary {
    pub campsite: i64,
    pub tent: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsStateDictionary {
    pub allowed: PermissionsDictionary,
    pub denied: PermissionsDictionary,
}
