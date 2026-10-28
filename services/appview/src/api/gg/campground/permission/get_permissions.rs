use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::{
    GetPermissionsOutput, PermissionViewDetailed,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, sql_types::Bool};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            PermissionConsts,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
    },
    views::permissions::permission_view_detailed,
    xrpc::{campsite::OneOfInfo, error::Result},
};

#[derive(FromForm)]
pub struct GetPermissionsQuery {
    #[allow(dead_code)]
    tent_id: Option<Uuid>,
    #[allow(dead_code)]
    category_id: Option<Uuid>,
    #[allow(dead_code)]
    bonfire_id: Option<String>,
    non_self: Option<bool>,
}

macro_rules! get_scoped_permissions {
    (campsite: $campsite: expr, bonfire: $bonfire: expr, category: $category: expr, tent: $tent: expr, member: $member: expr, non_self: $non_self: expr, permission_filter: $perm_filter: expr) => {{
        // Since it depends from where you want to get permissions
        expect_permission!(
            PermissionCheck::member(&$member)
                .is_owner(&$campsite)
                .skip_if_owner(|permission| scoped_permission(
                    permission.fetch_roles(),
                    &$campsite.id,
                    $bonfire,
                    $category,
                    $tent
                )
                .has_permissions(
                    PermissionConsts::VIEW_CONTENT | PermissionConsts::MANAGE_TENTS
                ))?
        );
        let mut conn = establish_connection().unwrap();

        let permissions = campsite_permission::table
            .filter(
                $perm_filter.and(
                    campsite_permission::roleid
                        .is_null()
                        .and(campsite_permission::userid.ne(&$member.user_id))
                        .or::<bool, Bool>(!$non_self),
                ),
            )
            .load::<CampsitePermission>(&mut conn)
            .map_err(handle_all_db_errors)?
            .iter()
            .map(permission_view_detailed)
            .collect::<Vec<PermissionViewDetailed>>();

        Ok(Json(GetPermissionsOutput { permissions }))
    }};
}

#[get("/xrpc/gg.campground.permission.getPermissions?<query..>", rank = 1)]
pub async fn get_permissions(
    auth: OneOfInfo<'_>,
    query: GetPermissionsQuery,
) -> Result<Json<GetPermissionsOutput>> {
    let non_self = query.non_self.unwrap_or(false);

    match auth {
        OneOfInfo::Tent(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.tent.bonfire_id,
                category: auth.tent.category_id.as_ref(),
                tent: Some(&auth.tent.id),
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::tentid.eq(auth.tent.id)
            )
        }
        OneOfInfo::Category(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.category.bonfire_id,
                category: Some(&auth.category.id),
                tent: None,
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::categoryid.eq(auth.category.id)
            )
        }
        OneOfInfo::Bonfire(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.bonfire.id,
                category: None,
                tent: None,
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::bonfireid.eq(auth.bonfire.id)
            )
        }
    }
}
