use appview_schema::{
    models::appview::{Actor, CampsiteBan, Profile},
    schema::appview::{self, campsite_ban},
};
use campground_lexicon::gg::campground::membership::GetMemberBansOutput;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
    },
    views::members::member_ban_view,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.membership.getMemberBans?<campsite_id>&<offset>&<limit>")]
pub async fn get_member_bans(
    auth: CampsiteInfo<'_>,
    campsite_id: &str,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<Json<GetMemberBansOutput>> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50);

    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest("Expected 'limit' query to be between (and including) 1 and 100, as well as 'offset' query to be positive integer or 0".to_string()));
    }

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::BAN_MEMBERS,
        0
    ));

    let mut conn = establish_connection().unwrap();

    let member_bans = campsite_ban::table
        .filter(campsite_ban::campsiteid.eq(campsite_id))
        .left_join(appview::profile::table.on(appview::profile::creator.eq(campsite_ban::userid)))
        .inner_join(appview::actor::table.on(appview::actor::did.eq(campsite_ban::userid)))
        .load::<(CampsiteBan, Option<Profile>, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(|x| member_ban_view(&x.0, x.1.as_ref(), &x.2))
        .collect();

    Ok(Json(GetMemberBansOutput { member_bans }))
}
