use appview_schema::{models::appview::{Actor, CampsiteBan, Profile}, schema::appview::{self, campsite_ban}};
use campground_lexicon::gg::campground::membership::CampsiteBanView;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_ban_view, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::event_next_campsite}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.deleteMemberBan?<campsite_id>&<actor>")]
pub async fn delete_member_ban(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, actor: &str) -> Result<Json<CampsiteBanView>> {    
    if actor == auth.actor.did {
        return Err(XRPCError::Forbidden("Member cannot delete ban from themselves".to_string()));
    } else if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::BAN_MEMBERS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let (member_ban, profile, target_actor) = campsite_ban::table
        .filter(
            campsite_ban::campsiteid
                .eq(campsite_id)
                .and(
                    campsite_ban::userid
                        .eq(actor)
                )
        )
        .left_join(
            appview::profile::table
                .on(
                    appview::profile::creator
                        .eq(campsite_ban::userid)
                )
        )
        .inner_join(
            appview::actor::table
                .on(
                    appview::actor::did
                        .eq(campsite_ban::userid)
                )
        )
        .first::<(CampsiteBan, Option<Profile>, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::delete(campsite_ban::table)
        .filter(
            campsite_ban::campsiteid
                .eq(campsite_id)
                .and(
                    campsite_ban::userid
                        .eq(actor)
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next_campsite(
        event_subject,
        &auth.campsite.id,
        CampsitePermissionConsts::BAN_MEMBERS,
        "MemberBanDeleted",
        campsite_ban_view(&member_ban, &profile, &target_actor)
    );

    Ok(Json(campsite_ban_view(&member_ban, &profile, &target_actor)))
}
