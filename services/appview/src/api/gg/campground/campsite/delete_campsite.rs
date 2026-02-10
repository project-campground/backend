use appview_schema::schema::appview;
use diesel::{ExpressionMethods, PgArrayExpressionMethods, RunQueryDsl, dsl::array_remove};

use crate::{database::establish_connection, helpers::api::handle_select_first_error, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.campsite.deleteCampsite?<campsite_id>")]
pub async fn delete_campsite(auth: CampsiteInfo<'_>, campsite_id: &str) -> Result<()> {    
    if auth.campsite.owner != auth.member.user_id {
        return Err(XRPCError::Forbidden("Actor is not the owner of the campsite".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    diesel::delete(appview::campsite::table)
        .filter(
            appview::campsite::id
                .eq(campsite_id)
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;
    
    diesel::update(
        appview::actor::table
    )
        .filter(
            appview::actor::campsites
                .contains(vec![ campsite_id ])
        )
        .set(
            appview::actor::campsites
            .eq(
                array_remove(appview::actor::campsites, campsite_id)
            )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;
    
    return Ok(());
}