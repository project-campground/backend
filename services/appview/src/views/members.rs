use appview_schema::models::appview::{Actor, CampsiteBan, CampsiteMember, Profile};
use campground_lexicon::gg::campground::{
    actor::ProfileViewBasic,
    membership::{MemberBanView, MemberViewAuthor, MemberViewBasic, MemberViewDetailed},
};
use uuid::Uuid;

use crate::views::{
    profiles::{
        profile_view_basic_deleted_actor, profile_view_basic_deleted_profile,
        profile_view_basic_from_db, profile_view_detailed_from_db,
    },
    util::serialize_datetime,
};

pub fn member_view_basic(
    campsite_member: &CampsiteMember,
    profile: &Profile,
    actor: &Actor,
) -> MemberViewBasic {
    return MemberViewBasic {
        user: profile_view_basic_from_db(actor, profile),
        nickname: campsite_member.nickname.clone(),
        roles: campsite_member
            .roles
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<Uuid>>(),
    };
}

pub fn member_view_detailed(
    campsite_member: &CampsiteMember,
    profile: &Profile,
    actor: &Actor,
) -> MemberViewDetailed {
    return MemberViewDetailed {
        campsite_id: campsite_member.campsite_id.clone(),
        used_invite_id: campsite_member.used_invite_id.clone(),
        user: profile_view_detailed_from_db(actor, profile),
        nickname: campsite_member.nickname.clone(),
        roles: campsite_member
            .roles
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<Uuid>>(),
        joined_at: serialize_datetime(campsite_member.joined_at),
    };
}

const DEFAULT_NO_ROLES: Vec<Uuid> = vec![];

pub fn member_view_author(
    campsite_member: &Option<CampsiteMember>,
    profile_view: ProfileViewBasic,
) -> MemberViewAuthor {
    return MemberViewAuthor {
        is_member: campsite_member.is_some(),
        user: profile_view,
        nickname: campsite_member
            .clone()
            .map(|member| member.nickname.clone())
            .flatten(),
        roles: campsite_member.clone().map_or(DEFAULT_NO_ROLES, |member| {
            member
                .roles
                .iter()
                .filter_map(|&x| x)
                .collect::<Vec<Uuid>>()
        }),
    };
}

pub fn created_by_view(
    created_by: &String,
    actor: &Option<Actor>,
    profile: &Option<Profile>,
    member: &Option<CampsiteMember>,
) -> MemberViewAuthor {
    let profile_view = match actor {
        None => profile_view_basic_deleted_actor(created_by.clone()),
        Some(x) => match profile {
            None => profile_view_basic_deleted_profile(x),
            Some(y) => profile_view_basic_from_db(x, y),
        },
    };

    member_view_author(member, profile_view)
}

pub fn member_ban_view(
    ban: &CampsiteBan,
    profile: &Option<Profile>,
    actor: &Actor,
) -> MemberBanView {
    return MemberBanView {
        user: match profile {
            Some(profile) => profile_view_basic_from_db(actor, profile),
            None => profile_view_basic_deleted_profile(actor),
        },
        user_id: ban.user_id.clone(),
        campsite_id: ban.campsite_id.clone(),
        reason: ban.reason.clone(),
        created_at: serialize_datetime(ban.created_at),
        created_by: ban.created_by.clone(),
        updated_at: serialize_datetime(ban.updated_at),
        updated_by: ban.updated_by.clone(),
    };
}
