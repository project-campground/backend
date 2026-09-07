use appview_schema::schema::appview;
use campground_lexicon::gg::campground::membership::CampsiteLeftOutput;
use diesel::{ExpressionMethods, PgArrayExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    helpers::ws::event_next_campsite_global,
    realtime::data::ReactiveSubject,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/gg.campground.campsite.deleteCampsite?<campsite_id>")]
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

    diesel::delete(appview::campsite::table)
        .filter(appview::campsite::id.eq(campsite_id))
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::actor::table)
        .filter(appview::actor::campsites.contains(vec![campsite_id]))
        .set(appview::actor::campsites.eq(diesel::dsl::array_remove(
            appview::actor::campsites,
            campsite_id,
        )))
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

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
