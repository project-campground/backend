use campground_lexicon::chat::campground::member::MemberViewDetailed;
use cds_schema::{
    models::cds::{Actor, CampsiteMember, Profile},
    schema::cds::{campsite_member, profile},
};
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
    "/xrpc/chat.campground.member.updateMember?<campsite_id>&<actor>",
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

    let (member, actor_db, profile) = crate::schema::cds::campsite_member::table
        .filter(
            crate::schema::cds::campsite_member::campsite
                .eq(campsite_id)
                .and(crate::schema::cds::campsite_member::actor.eq(actor)),
        )
        .inner_join(
            crate::schema::cds::actor::table
                .on(crate::schema::cds::actor::did.eq(campsite_member::actor)),
        )
        .left_join(profile::table.on(profile::actor.eq(campsite_member::actor)))
        .first::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let updated_members = diesel::update(campsite_member::table)
        .filter(
            campsite_member::campsite
                .eq(campsite_id)
                .and(campsite_member::actor.eq(&member.actor)),
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
