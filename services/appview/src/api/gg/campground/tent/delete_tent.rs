#![allow(unused_variables)]
use appview_schema::schema::appview::{campsite_permission, tent};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        permissions::{
            ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner,
        },
        ws::event_next_tent,
    },
    realtime::data::ReactiveSubject,
    views::tents::tent_view_basic,
    xrpc::{
        campsite::TentInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/gg.campground.tent.deleteTent?<tent_id>")]
pub async fn delete_tent(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    tent_id: &str,
) -> Result<Json<TentViewBasic>> {
    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.tent.bonfire_id,
        auth.tent.category_id.clone(),
        Some(auth.tent.id),
        &auth.member,
        GeneralPermissionConsts::MANAGE_TENTS,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    let mut conn = establish_connection().unwrap();

    diesel::delete(tent::table)
        .filter(tent::id.eq(auth.tent.id))
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;
    diesel::delete(campsite_permission::table)
        .filter(campsite_permission::tentid.eq(auth.tent.id))
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    let view = tent_view_basic(&auth.tent);
    event_next_tent(event_subject, &auth.tent, true, "TentDeleted", &view);

    return Ok(Json(view));
}
