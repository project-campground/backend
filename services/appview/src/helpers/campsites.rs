use appview_schema::models::appview::{Actor, Bonfire, Campsite, CampsiteMember, Profile};
use campground_lexicon::gg::campground::{campsite::{BonfireViewBasic, BonfireViewDetailed, CampsiteMemberViewBasic, CampsiteViewBasic, CampsiteViewDetailed}, tent::{TentCategoryView, TentViewBasic}};

use crate::helpers::{util::serialize_datetime, views::profile_view_basic_from_db};

pub fn campsite_view_basic(campsite: &Campsite) -> CampsiteViewBasic {
    return CampsiteViewBasic {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags
            .iter()
            .filter_map(|x| x.clone())
            .collect(),
        member_count: campsite.member_dids.len(),
        created_by: campsite.created_by.clone(),
        created_at: serialize_datetime(campsite.created_at),
        updated_by: campsite.updated_by.clone(),
        updated_at: serialize_datetime(campsite.updated_at)
    };
}

pub fn campsite_view_detailed(campsite: &Campsite, bonfires: Vec<BonfireViewBasic>) -> CampsiteViewDetailed {
    return CampsiteViewDetailed {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags
            .iter()
            .filter_map(|x| x.clone())
            .collect(),
        member_count: campsite.member_dids.len(),
        created_by: campsite.created_by.clone(),
        created_at: serialize_datetime(campsite.created_at),
        updated_by: campsite.updated_by.clone(),
        updated_at: serialize_datetime(campsite.updated_at),
        bonfires: bonfires,
    };
}

pub fn campsite_member_view_basic(campsite_member: &CampsiteMember, profile: &Profile, actor: &Actor) -> CampsiteMemberViewBasic {
    return CampsiteMemberViewBasic {
        user: profile_view_basic_from_db(actor, profile),
        user_id: campsite_member.user_id.clone(),
        campsite_id: campsite_member.campsite_id.clone(),
        joined_at: serialize_datetime(campsite_member.joined_at),
        nickname: campsite_member.nickname.clone(),
    };
}

// pub fn campsite_member_view_detailed(campsite_member: &CampsiteMember) -> CampsiteMemberViewDetailed {
//     return CampsiteMemberViewDetailed {
//         user_id: campsite_member.user_id.clone(),
//         campsite_id: campsite_member.campsite_id.clone(),
//         joined_at: serialize_datetime(campsite_member.joined_at),
//         used_invite_id: campsite_member.used_invite_id.clone(),
//         nickname: campsite_member.nickname.clone(),
//     };
// }

pub fn bonfire_view_basic(bonfire: &Bonfire) -> BonfireViewBasic {
    return BonfireViewBasic {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        priority: bonfire.priority,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at)
    };
}
pub fn bonfire_view_detailed(bonfire: &Bonfire, tents: Vec<TentViewBasic>, categories: Vec<TentCategoryView>) -> BonfireViewDetailed {
    return BonfireViewDetailed {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        priority: bonfire.priority,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at),
        tents,
        categories
    };
}
