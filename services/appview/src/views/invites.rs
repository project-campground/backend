use appview_schema::models::appview::{Actor, Campsite, CampsiteInvite, Profile};
use campground_lexicon::gg::campground::invite::{
    CampsiteInviteViewCampsite, CampsiteInviteViewGlobal,
};

use crate::views::{
    campsites::campsite_view_basic, profiles::profile_view_basic, util::serialize_datetime,
};

pub fn campsite_invite_view_campsite(
    campsite_invite: &CampsiteInvite,
    profile: Option<&Profile>,
    actor: &Actor,
) -> CampsiteInviteViewCampsite {
    return CampsiteInviteViewCampsite {
        id: campsite_invite.id.clone(),
        expires_at: campsite_invite.expires_at.map(|x| serialize_datetime(x)),
        allowed_amount: campsite_invite.allowed_amount.clone(),
        used: campsite_invite.used,
        created_at: serialize_datetime(campsite_invite.created_at),
        created_by: profile_view_basic(actor, profile),
    };
}

pub fn campsite_invite_view_global(
    campsite_invite: &CampsiteInvite,
    campsite: &Campsite,
    profile: Option<&Profile>,
    actor: &Actor,
) -> CampsiteInviteViewGlobal {
    return CampsiteInviteViewGlobal {
        id: campsite_invite.id.clone(),
        campsite: campsite_view_basic(campsite),
        expires_at: campsite_invite.expires_at.map(|x| serialize_datetime(x)),
        allowed_amount: campsite_invite.allowed_amount.clone(),
        used: campsite_invite.used,
        created_at: serialize_datetime(campsite_invite.created_at),
        created_by: profile_view_basic(actor, profile),
    };
}
