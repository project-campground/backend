#![
    allow(unused_variables)
]
use appview_schema::schema::appview::{campsite_permission, tent};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection, helpers::{permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}, tents::tent_view_basic, ws::event_next_tent}, realtime::data::ReactiveSubject, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteTent?<tent_id>")]
pub async fn delete_tent(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, tent_id: &str) -> Result<Json<TentViewBasic>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, CampsitePermissionConsts::MANAGE_TENTS, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    diesel::delete(tent::table)
        .filter(
            tent::id
                .eq(auth.tent.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;
    diesel::delete(campsite_permission::table)
        .filter(
            campsite_permission::tentid
                .eq(auth.tent.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    event_next_tent(event_subject, &auth.tent, true, "TentDeleted", tent_view_basic(&auth.tent));

    return Ok(Json(tent_view_basic(&auth.tent)));
}
