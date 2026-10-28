use campground_lexicon::gg::campground::tent::TentViewDetailed;
use rocket::serde::json::Json;

use crate::{
    expect_permission,
    helpers::permissions::{
        PermissionFlag,
        check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
    },
    views::tents::tent_view_detailed,
    xrpc::{campsite::TentInfo, error::Result},
};

#[allow(unused_variables)]
#[get("/xrpc/gg.campground.tent.getTent?<tent_id>")]
pub async fn get_tent(auth: TentInfo<'_>, tent_id: &str) -> Result<Json<TentViewDetailed>> {
    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &auth.campsite.id,
                &auth.tent.bonfire_id,
                auth.tent.category_id.as_ref(),
                Some(&auth.tent.id)
            )
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_TENTS))?
    );

    return Ok(Json(tent_view_detailed(&auth.tent)));
}
