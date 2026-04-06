use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::gg::campground::{actor::ProfileViewBasic, campsite::CampsiteViewBasic};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteInviteViewCampsite {
    pub id: Uuid,
    pub allowed_amount: Option<i32>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub created_by: ProfileViewBasic,
    pub used: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteInviteViewGlobal {
    pub id: Uuid,
    pub campsite: CampsiteViewBasic,
    pub allowed_amount: Option<i32>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub created_by: ProfileViewBasic,
    pub used: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCampsiteInvitesOutput {
    pub invites: Vec<CampsiteInviteViewCampsite>,
}
