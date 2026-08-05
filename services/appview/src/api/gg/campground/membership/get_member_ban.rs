use appview_schema::{
    models::appview::{Actor, CampsiteBan, Profile},
    schema::appview::{self, campsite_ban},
};
use campground_lexicon::gg::campground::membership::MemberBanView;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
    },
    views::members::member_ban_view,
    xrpc::{campsite::CampsiteInfo, error::Result},
};

#[get("/xrpc/gg.campground.membership.getMemberBan?<campsite_id>&<actor>")]
pub async fn get_member_ban(
    auth: CampsiteInfo<'_>,
    campsite_id: &str,
    actor: &str,
) -> Result<Json<MemberBanView>> {
    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::BAN_MEMBERS,
        0
    ));

    let mut conn = establish_connection().unwrap();

    let member_ban = campsite_ban::table
        .filter(
            campsite_ban::campsiteid
                .eq(campsite_id)
                .and(campsite_ban::userid.eq(actor)),
        )
        .left_join(appview::profile::table.on(appview::profile::creator.eq(campsite_ban::userid)))
        .inner_join(appview::actor::table.on(appview::actor::did.eq(campsite_ban::userid)))
        .first::<(CampsiteBan, Option<Profile>, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(Json(member_ban_view(
        &member_ban.0,
        member_ban.1.as_ref(),
        &member_ban.2,
    )))
}
