use campground_lexicon::chat::campground::member::MemberBanView;
use cds_schema::{
    models::cds::{Actor, CampsiteBan, Profile},
    schema::cds::{self, member_ban},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable},
        },
        ws::event_next_campsite,
    },
    realtime::data::ReactiveSubject,
    views::members::member_ban_view,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/chat.campground.member.deleteMemberBan?<campsite_id>&<actor>")]
pub async fn delete_member_ban(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    actor: &str,
) -> Result<Json<MemberBanView>> {
    if actor == auth.actor.did {
        return Err(XRPCError::Forbidden(
            "Member cannot delete ban from themselves".to_string(),
        ));
    }

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| permission
                .fetch_roles()
                .has_permissions(PermissionFlag::CREATE_INVITES))?
    );

    let mut conn = establish_connection().unwrap();

    let (member_ban, profile, target_actor) = member_ban::table
        .filter(
            member_ban::campsite
                .eq(campsite_id)
                .and(member_ban::target.eq(actor)),
        )
        .left_join(cds::profile::table.on(cds::profile::actor.eq(member_ban::target)))
        .inner_join(cds::actor::table.on(cds::actor::did.eq(member_ban::target)))
        .first::<(CampsiteBan, Option<Profile>, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::delete(member_ban::table)
        .filter(
            member_ban::campsite
                .eq(campsite_id)
                .and(member_ban::target.eq(actor)),
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    let view = member_ban_view(&member_ban, profile.as_ref(), &target_actor);

    event_next_campsite(
        event_subject,
        &auth.campsite.id,
        PermissionFlag::BAN_MEMBERS,
        "MemberBanDeleted",
        &view,
    );

    Ok(Json(view))
}
