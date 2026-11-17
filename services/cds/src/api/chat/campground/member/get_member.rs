use campground_lexicon::chat::campground::member::MemberViewDetailed;
use cds_schema::{
    models::cds::{Actor, CampsiteMember, Profile},
    schema::cds::{campsite_member, profile},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    views::members::member_view_detailed,
    xrpc::{campsite::CampsiteInfoBasic, error::Result},
};

#[get("/xrpc/chat.campground.member.getMember?<campsite_id>&<actor>")]
pub async fn get_member(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
    actor: &str,
) -> Result<Json<MemberViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let member = crate::schema::cds::campsite_member::table
        .filter(
            crate::schema::cds::campsite_member::campsite
                .eq(campsite_id)
                .and(crate::schema::cds::campsite_member::actor.eq(actor)),
        )
        .inner_join(
            crate::schema::cds::actor::table
                .on(crate::schema::cds::actor::did.eq(campsite_member::actor)),
        )
        .left_join(profile::table.on(profile::actor.eq(campsite_member::actor)))
        .first::<(CampsiteMember, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(member_view_detailed(
        &member.0,
        member.2.as_ref(),
        &member.1,
    )));
}
