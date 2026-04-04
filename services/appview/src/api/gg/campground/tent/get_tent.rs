use campground_lexicon::gg::campground::tent::TentViewDetailed;
use rocket::serde::json::Json;

use crate::{
    expect_permission,
    helpers::permissions::{ContentPermissionConsts, has_leveled_perms_or_owner},
    views::tents::tent_view_detailed,
    xrpc::{campsite::TentInfo, error::Result},
};

#[allow(unused_variables)]
#[get("/xrpc/gg.campground.tent.getTent?<tent_id>")]
pub async fn get_tent(auth: TentInfo<'_>, tent_id: &str) -> Result<Json<TentViewDetailed>> {
    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.tent.bonfire_id,
        auth.tent.category_id.clone(),
        Some(auth.tent.id),
        &auth.member,
        0,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    return Ok(Json(tent_view_detailed(&auth.tent)));
}
