use appview_schema::{
    models::appview::{Actor, CampsiteMember, Profile},
    schema::appview::{campsite_member, profile},
};
use campground_lexicon::gg::campground::membership::{GetMembersOutput, MemberViewDetailed};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            PermissionConsts,
            check::{PermissionCheck, PermissionCheckTestable},
        },
    },
    views::members::member_view_detailed,
    xrpc::{campsite::CampsiteInfo, error::Result},
};

#[get("/xrpc/gg.campground.membership.getMembersDetailed?<campsite_id>&<limit>&<offset>")]
pub async fn get_members_detailed(
    auth: CampsiteInfo<'_>,
    campsite_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Json<GetMembersOutput<MemberViewDetailed>>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| permission.fetch_roles().has_any_permissions(
                PermissionConsts::BAN_MEMBERS
                    | PermissionConsts::KICK_MEMBERS
                    | PermissionConsts::GIVE_ROLES
                    | PermissionConsts::MUTE_MEMBERS
            ))?
    );

    let mut conn = establish_connection().unwrap();

    let members = crate::schema::appview::campsite_member::table
        .filter(crate::schema::appview::campsite_member::campsiteid.eq(campsite_id))
        .limit(limit)
        .offset(offset)
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .load::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|a| member_view_detailed(&a.0, a.2.as_ref(), &a.1))
        .collect::<Vec<MemberViewDetailed>>();

    return Ok(Json(GetMembersOutput::<MemberViewDetailed> { members }));
}
