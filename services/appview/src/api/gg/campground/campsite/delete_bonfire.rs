use appview_schema::{models::appview::Bonfire, schema::appview};
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::bonfire_view_basic, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.campsite.deleteBonfire?<campsite_id>&<bonfire_id>")]
pub async fn delete_bonfire(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, bonfire_id: &str) -> Result<Json<BonfireViewBasic>> {    
    let mut conn = establish_connection().unwrap();

    let existing_bonfires = appview::bonfire::table
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
        )
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    if existing_bonfires.len() < 2 {
        return Err(XRPCError::Forbidden("Cannot delete last bonfire".to_string()));
    }

    let bonfire = existing_bonfires.iter().find(|x| x.id == bonfire_id);

    if bonfire.is_none() {
        return Err(XRPCError::NotFound);
    } else if !has_tent_perms_or_owner(&auth.campsite, &bonfire_id, None, None, &auth.member, CampsitePermissionConsts::MANAGE_BONFIRES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    diesel::delete(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(bonfire_id)
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    let bonfire = bonfire.unwrap();

    event_next(event_subject, &auth.campsite.id, "BonfireDeleted", bonfire_view_basic(bonfire));

    return Ok(Json(bonfire_view_basic(bonfire)));
}