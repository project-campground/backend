use appview_schema::{models::appview::{CampsitePermission, Tent}, schema::appview::{self, campsite_permission}};
use campground_lexicon::gg::campground::{campsite::CampsitePermissionView, tent::TentViewDetailed};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_permission_view, tents::tent_view_detailed}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getTent?<tent_id>")]
pub async fn get_tent(auth: Authorization<'_>, tent_id: &str) -> Result<Json<TentViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(tent_id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'tent_id' format. Expected UUID".to_string()))?;

    let tent_with_perms: Vec<(Tent, Option<CampsitePermission>)> = appview::tent::table
        .filter(
            appview::tent::id
                .eq(uuid)
        )
        .left_join(
            campsite_permission::table
                .on(
                    campsite_permission::bonfireid
                        .is_not_null()
                        .and(
                            campsite_permission::bonfireid
                                .assume_not_null()
                                .eq(appview::tent::bonfireid)
                        )
                        .or(
                            campsite_permission::categoryid
                                .eq(appview::tent::categoryid)
                                .and(
                                    appview::tent::categoryid.is_not_null()
                                )
                        )
                        .or(
                            campsite_permission::tentid
                                .is_not_null()
                                .and(
                                    appview::tent::id.eq(campsite_permission::tentid.assume_not_null())
                                )
                        )
                        .and(
                            
                            campsite_permission::userid
                                .is_null()
                                .or(
                                    campsite_permission::userid
                                        .eq(auth.actor_did.clone())
                                )
                        )
                )
        )
        .load::<(Tent, Option<CampsitePermission>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let tent = tent_with_perms.first().map(|x| x.0.clone()).ok_or(XRPCError::NotFound)?;
    let permissions = tent_with_perms
        .iter()
        .filter_map(|x| x.1.clone())
        .map(campsite_permission_view)
        .collect::<Vec<CampsitePermissionView>>();

    return Ok(Json(tent_view_detailed(&tent, permissions)));
}