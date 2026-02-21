use appview_schema::{models::appview::{Actor, CampsiteMember, Profile}, schema::appview::{campsite_member, profile}};
use campground_lexicon::gg::campground::membership::{CampsiteMemberViewDetailed, GetMembersDetailedOutput};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection, helpers::{api::handle_all_db_errors, campsites::campsite_member_view_detailed}, xrpc::{
        campsite::CampsiteInfoBasic, error::Result
    }
};

#[get("/xrpc/gg.campground.membership.getMembersDetailed?<campsite_id>&<limit>&<offset>")]
pub async fn get_members_detailed(_auth: CampsiteInfoBasic<'_>, campsite_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetMembersDetailedOutput>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let mut conn = establish_connection().unwrap();

    let members = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
        )
        .limit(limit)
        .offset(offset)
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
        .load::<(CampsiteMember, Profile, Actor)>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|a| campsite_member_view_detailed(&a.0, &a.1, &a.2))
        .collect::<Vec<CampsiteMemberViewDetailed>>();

    return Ok(Json(GetMembersDetailedOutput { members }));
}
