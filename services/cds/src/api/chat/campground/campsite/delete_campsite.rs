use campground_lexicon::chat::campground::member::CampsiteLeftOutput;
use cds_schema::schema::cds;
use diesel::{Connection, ExpressionMethods, PgArrayExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    helpers::{api::handle_all_db_errors, ws::event_next_campsite_global},
    realtime::data::ReactiveSubject,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/chat.campground.campsite.deleteCampsite?<campsite_id>")]
pub async fn delete_campsite(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
) -> Result<Json<CampsiteLeftOutput>> {
    if auth.campsite.owner != auth.member.user_id {
        return Err(XRPCError::Forbidden(
            "Actor is not the owner of the campsite".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    conn.transaction(|conn| {
        diesel::delete(cds::campsite::table)
            .filter(cds::campsite::id.eq(campsite_id))
            .execute(conn)?;

        diesel::update(cds::actor::table)
            .filter(cds::actor::campsites.contains(vec![campsite_id]))
            .set(cds::actor::campsites.eq(diesel::dsl::array_remove(
                cds::actor::campsites,
                campsite_id,
            )))
            .execute(conn)
    })
    .map_err(handle_all_db_errors)?;

    event_next_campsite_global(
        event_subject,
        &auth.campsite.id,
        "CampsiteLeft",
        &CampsiteLeftOutput {
            id: campsite_id.to_string(),
        },
    );

    return Ok(Json(CampsiteLeftOutput {
        id: campsite_id.to_string(),
    }));
}
