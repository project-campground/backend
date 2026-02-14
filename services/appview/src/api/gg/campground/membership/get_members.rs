use appview_schema::{models::appview::{Actor, CampsiteMember, Profile}, schema::appview::{campsite_member, profile}};
use campground_lexicon::gg::campground::membership::{CampsiteMemberViewBasic, GetMembersOutput};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_member_view_basic, deduplicate_list, lower_list}, xrpc::{
        campsite::CampsiteInfoBasic, error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.membership.getMembers?<campsite_id>&<limit>&<offset>", rank = 1)]
pub async fn get_members_any(_auth: CampsiteInfoBasic<'_>, campsite_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetMembersOutput>> {
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
        .expect("Error loading members")
        .iter()
        .map(|a| campsite_member_view_basic(&a.0, &a.1, &a.2))
        .collect::<Vec<CampsiteMemberViewBasic>>();

    return Ok(Json(GetMembersOutput { members }));
}

#[get("/xrpc/gg.campground.campsite.getMembers?<campsite_id>&<actors>", rank = 2)]
pub async fn get_members_given(_auth: CampsiteInfoBasic<'_>, campsite_id: &str, actors: Vec<&str>) -> Result<Json<GetMembersOutput>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest("actors query must have at least 1 actor and less than or equal to 25".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let members = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::campsite_member::userid
                        .eq_any(actors)
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
        .load::<(CampsiteMember, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(|a| campsite_member_view_basic(&a.0, &a.1, &a.2))
        .collect::<Vec<CampsiteMemberViewBasic>>();

    return Ok(Json(GetMembersOutput { members }));
}