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
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
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
        GeneralPermissionConsts::MANAGE_SELF_IDENTITY
    } else {
        GeneralPermissionConsts::MANAGE_OTHERS_IDENTITY
    };

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        required_permission,
        0
    ));

    let (member, profile, actor) = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::campsite_member::userid.eq(actor)),
        )
        .inner_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .select((
            campsite_member::all_columns,
            profile::all_columns,
            crate::schema::appview::actor::all_columns,
        ))
        .first::<(CampsiteMember, Profile, Actor)>(&mut conn)
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
        &profile,
        &actor,
    )));
}
