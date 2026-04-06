use appview_schema::models::appview::Campsite;
use campground_lexicon::gg::campground::{
    bonfire::BonfireViewBasic,
    campsite::{CampsiteViewBasic, CampsiteViewDetailed},
    membership::MemberViewBasic,
    role::RoleViewBasic,
};

use crate::views::util::serialize_datetime;

pub fn campsite_view_basic(campsite: &Campsite) -> CampsiteViewBasic {
    return CampsiteViewBasic {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags.iter().filter_map(|x| x.clone()).collect(),
        member_count: campsite.member_dids.len(),
        owner: campsite.owner.clone(),
    };
}

pub fn campsite_view_detailed(
    campsite: &Campsite,
    bonfires: Vec<BonfireViewBasic>,
    roles: Vec<RoleViewBasic>,
    member: MemberViewBasic,
) -> CampsiteViewDetailed {
    return CampsiteViewDetailed {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags.iter().filter_map(|x| x.clone()).collect(),
        member_count: campsite.member_dids.len(),
        owner: campsite.owner.clone(),
        created_by: campsite.created_by.clone(),
        created_at: serialize_datetime(campsite.created_at),
        updated_by: campsite.updated_by.clone(),
        updated_at: serialize_datetime(campsite.updated_at),
        bonfires: bonfires,
        roles: roles,
        me: member,
    };
}
