use appview_schema::{
    models::appview::{Actor, CampsiteBan, Profile},
    schema::appview::{self, campsite_ban},
};
use campground_lexicon::gg::campground::membership::MemberBanView;
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

#[post("/xrpc/gg.campground.membership.deleteMemberBan?<campsite_id>&<actor>")]
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

    let (member_ban, profile, target_actor) = campsite_ban::table
        .filter(
            campsite_ban::campsiteid
                .eq(campsite_id)
                .and(campsite_ban::userid.eq(actor)),
        )
        .left_join(appview::profile::table.on(appview::profile::creator.eq(campsite_ban::userid)))
        .inner_join(appview::actor::table.on(appview::actor::did.eq(campsite_ban::userid)))
        .first::<(CampsiteBan, Option<Profile>, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::delete(campsite_ban::table)
        .filter(
            campsite_ban::campsiteid
                .eq(campsite_id)
                .and(campsite_ban::userid.eq(actor)),
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
