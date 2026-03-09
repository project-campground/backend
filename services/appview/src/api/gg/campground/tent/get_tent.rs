use campground_lexicon::gg::campground::tent::TentViewDetailed;
use rocket::serde::json::Json;

use crate::{
    helpers::{permissions::{TentPermissionConsts, has_tent_perms_or_owner}, tents::tent_view_detailed}, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[allow(unused_variables)]
#[get("/xrpc/gg.campground.tent.getTent?<tent_id>")]
pub async fn get_tent(auth: TentInfo<'_>, tent_id: &str) -> Result<Json<TentViewDetailed>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, 0, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    return Ok(Json(tent_view_detailed(&auth.tent)));
}