use appview_schema::{models::appview::{Actor, CampsiteMember, Profile}, schema::appview::{campsite_member, profile}};
use campground_lexicon::gg::campground::membership::CampsiteMemberViewDetailed;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_member_view_detailed}, xrpc::{
        campsite::CampsiteInfoBasic, error::Result
    }
};

#[get("/xrpc/gg.campground.membership.getMember?<campsite_id>&<actor>")]
pub async fn get_member(_auth: CampsiteInfoBasic<'_>, campsite_id: &str, actor: &str) -> Result<Json<CampsiteMemberViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let member = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::campsite_member::userid
                        .eq(actor)
                )
        )
        .inner_join(
            profile::table
                .on(
                    profile::creator.eq(
                        campsite_member::userid
                    )
                )
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(
                    crate::schema::appview::actor::did.eq(
                        campsite_member::userid
                    )
                )
        )
        .select(
            (campsite_member::all_columns, profile::all_columns, crate::schema::appview::actor::all_columns)
        )
        .first::<(CampsiteMember, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_member_view_detailed(&member.0, &member.1, &member.2)));
}
