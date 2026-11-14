use campground_lexicon::chat::campground::member::{GetMembersOutput, MemberViewBasic};
use cds_schema::{
    models::cds::{Actor, CampsiteMember, Profile},
    schema::cds::{campsite_member, profile},
};
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
    "/xrpc/chat.campground.member.getMembers?<campsite_id>&<limit>&<offset>",
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

    let members = crate::schema::cds::campsite_member::table
        .filter(crate::schema::cds::campsite_member::campsite.eq(campsite_id))
        .limit(limit)
        .offset(offset)
        .inner_join(
            crate::schema::cds::actor::table
                .on(crate::schema::cds::actor::did.eq(campsite_member::actor)),
        )
        .left_join(profile::table.on(profile::actor.eq(campsite_member::actor)))
        .load::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .expect("Error loading members")
        .iter()
        .map(|a| member_view_basic(&a.0, a.2.as_ref(), &a.1))
        .collect::<Vec<MemberViewBasic>>();

    return Ok(Json(GetMembersOutput::<MemberViewBasic> { members }));
}

#[get(
    "/xrpc/chat.campground.campsite.getMembers?<campsite_id>&<actors>",
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

    let members = crate::schema::cds::campsite_member::table
        .filter(
            crate::schema::cds::campsite_member::campsite
                .eq(campsite_id)
                .and(crate::schema::cds::campsite_member::actor.eq_any(actors)),
        )
        .inner_join(
            crate::schema::cds::actor::table
                .on(crate::schema::cds::actor::did.eq(campsite_member::actor)),
        )
        .left_join(profile::table.on(profile::actor.eq(campsite_member::actor)))
        .load::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(|a| member_view_basic(&a.0, a.2.as_ref(), &a.1))
        .collect::<Vec<MemberViewBasic>>();

    return Ok(Json(GetMembersOutput::<MemberViewBasic> { members }));
}
