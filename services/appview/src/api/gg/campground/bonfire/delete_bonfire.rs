use appview_schema::schema::appview;
use campground_lexicon::gg::campground::bonfire::BonfireViewBasic;
use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
        ws::event_next_bonfire,
    },
    realtime::data::ReactiveSubject,
    views::bonfires::bonfire_view_basic,
    xrpc::{campsite::BonfireInfo, error::Result},
};

#[post("/xrpc/gg.campground.bonfire.deleteBonfire?<bonfire_id>")]
pub async fn delete_bonfire(
    auth: BonfireInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    bonfire_id: &str,
) -> Result<Json<BonfireViewBasic>> {
    let mut conn = establish_connection().unwrap();

    if auth.bonfire.home {
        return Err(crate::xrpc::error::XRPCError::BadRequest(
            "Can't delete home bonfire".to_string(),
        ));
    }

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &auth.campsite.id,
                bonfire_id,
                None,
                None
            )
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_BONFIRES))?
    );

    diesel::delete(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(&auth.bonfire.campsite_id)
                .and(appview::bonfire::id.eq(bonfire_id)),
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    let view = bonfire_view_basic(&auth.bonfire);
    event_next_bonfire(event_subject, &auth.bonfire, true, "BonfireDeleted", &view);

    return Ok(Json(view));
}
