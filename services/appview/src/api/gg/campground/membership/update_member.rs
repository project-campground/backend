use appview_schema::{
    models::appview::{Actor, CampsiteMember, Profile},
    schema::appview::{campsite_member, profile},
};
use campground_lexicon::gg::campground::membership::MemberViewDetailed;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable},
        },
    },
    views::members::member_view_detailed,
    xrpc::{campsite::CampsiteInfo, error::Result},
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateMemberBody {
    nickname: String,
}

#[post(
    "/xrpc/gg.campground.membership.updateMember?<campsite_id>&<actor>",
    data = "<body>"
)]
pub async fn update_member(
    auth: CampsiteInfo<'_>,
    campsite_id: &str,
    actor: &str,
    body: Json<UpdateMemberBody>,
) -> Result<Json<MemberViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let required_permission = if auth.actor.did == actor {
        PermissionFlag::MANAGE_SELF_IDENTITY
    } else {
        PermissionFlag::MANAGE_OTHERS_IDENTITY
    };

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| permission
                .fetch_roles()
                .has_permissions(required_permission))?
    );

    let (member, actor_db, profile) = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::campsite_member::userid.eq(actor)),
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .first::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let updated_members = diesel::update(campsite_member::table)
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq(&member.user_id)),
        )
        .set((
            // Stuff changed
            campsite_member::nickname.eq(&body.nickname),
        ))
        .load::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let updated_member = updated_members.first().unwrap();

    return Ok(Json(member_view_detailed(
        &updated_member,
        profile.as_ref(),
        &actor_db,
    )));
}
