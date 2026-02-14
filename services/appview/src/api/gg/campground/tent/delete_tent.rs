#![
    allow(unused_variables)
]
use appview_schema::schema::appview::{campsite_permission, tent};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection, helpers::{permissions::{CampsitePermissionConsts, has_tent_perms_or_owner}, tents::tent_view_basic, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteTent?<tent_id>")]
pub async fn delete_tent(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, tent_id: &str) -> Result<Json<TentViewBasic>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, CampsitePermissionConsts::MANAGE_TENTS, 0).await? {
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

    event_next(event_subject, &auth.campsite.id, "TentDeleted", tent_view_basic(&auth.tent));

    return Ok(Json(tent_view_basic(&auth.tent)));
}
