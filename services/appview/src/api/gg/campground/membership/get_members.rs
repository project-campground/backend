use appview_schema::{
    models::appview::{Actor, CampsiteMember, Profile},
    schema::appview::{campsite_member, profile},
};
use campground_lexicon::gg::campground::membership::{GetMembersOutput, MemberViewBasic};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    helpers::{deduplicate_list, lower_list},
    views::members::member_view_basic,
    xrpc::{
        campsite::CampsiteInfoBasic,
        error::{Result, XRPCError},
    },
};

#[get(
    "/xrpc/gg.campground.membership.getMembers?<campsite_id>&<limit>&<offset>",
    rank = 1
)]
pub async fn get_members_any(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Json<GetMembersOutput<MemberViewBasic>>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let mut conn = establish_connection().unwrap();

    let members = crate::schema::appview::campsite_member::table
        .filter(crate::schema::appview::campsite_member::campsiteid.eq(campsite_id))
        .limit(limit)
        .offset(offset)
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .load::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .expect("Error loading members")
        .iter()
        .map(|a| member_view_basic(&a.0, a.2.as_ref(), &a.1))
        .collect::<Vec<MemberViewBasic>>();

    return Ok(Json(GetMembersOutput::<MemberViewBasic> { members }));
}

#[get(
    "/xrpc/gg.campground.campsite.getMembers?<campsite_id>&<actors>",
    rank = 2
)]
pub async fn get_members_given(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
    actors: Vec<&str>,
) -> Result<Json<GetMembersOutput<MemberViewBasic>>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest(
            "actors query must have at least 1 actor and less than or equal to 25".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    let members = crate::schema::appview::campsite_member::table
        .filter(
            crate::schema::appview::campsite_member::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::campsite_member::userid.eq_any(actors)),
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_member::userid)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .load::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(|a| member_view_basic(&a.0, a.2.as_ref(), &a.1))
        .collect::<Vec<MemberViewBasic>>();

    return Ok(Json(GetMembersOutput::<MemberViewBasic> { members }));
}
