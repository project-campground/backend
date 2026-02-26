use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::campsite::{CampsitePermissionView, GetCampsitePermissionsOutput};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, sql_types::Bool};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_all_db_errors, campsites::campsite_permission_view, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}}, xrpc::{
    campsite::{BonfireInfo, CategoryInfo, OneOfInfo, TentInfo}, error::{Result, XRPCError}
}};

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

#[get("/xrpc/gg.campground.permission.getPermissions?<query..>", rank = 1)]
pub async fn get_permissions(auth: OneOfInfo<'_>, query: GetPermissionsQuery) -> Result<Json<GetCampsitePermissionsOutput>> {
    let non_self = query.non_self.unwrap_or(false);

    match auth {
        OneOfInfo::Tent(tent_auth) =>
            get_tent_permissions(tent_auth, non_self).await,
        OneOfInfo::Category(category_auth) =>
            get_category_permissions(category_auth, non_self).await,
        OneOfInfo::Bonfire(bonfire_auth) =>
            get_bonfire_permissions(bonfire_auth, non_self).await,
    }
}

pub async fn get_tent_permissions(auth: TentInfo<'_>, non_self: bool) -> Result<Json<GetCampsitePermissionsOutput>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id.clone()), &auth.member, CampsitePermissionConsts::MANAGE_ROLES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let permissions = campsite_permission::table
        .filter(
            campsite_permission::tentid
                .eq(auth.tent.id)
                .and(
                    campsite_permission::roleid
                        .is_null()
                        .and(
                            campsite_permission::userid
                            .ne(&auth.actor.did)
                        )
                        .or::<bool, Bool>(
                            !non_self
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|x| campsite_permission_view(x))
        .collect::<Vec<CampsitePermissionView>>();

    Ok(Json(GetCampsitePermissionsOutput { permissions }))
}

pub async fn get_category_permissions(auth: CategoryInfo<'_>, non_self: bool) -> Result<Json<GetCampsitePermissionsOutput>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let mut conn = establish_connection().unwrap();

    let permissions = campsite_permission::table
        .filter(
            campsite_permission::categoryid
                .eq(auth.category.id)
                .and(
                    campsite_permission::roleid
                        .is_null()
                        .and(
                            campsite_permission::userid
                            .ne(&auth.actor.did)
                        )
                        .or::<bool, Bool>(
                            !non_self
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|x| campsite_permission_view(x))
        .collect::<Vec<CampsitePermissionView>>();

    Ok(Json(GetCampsitePermissionsOutput { permissions }))
}

pub async fn get_bonfire_permissions(auth: BonfireInfo<'_>, non_self: bool) -> Result<Json<GetCampsitePermissionsOutput>> {    
    if !has_tent_perms_or_owner(&auth.campsite, &auth.bonfire.id, None, None, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let mut conn = establish_connection().unwrap();

    let permissions = campsite_permission::table
        .filter(
            campsite_permission::bonfireid
                .eq(&auth.bonfire.id)
                .and(
                    campsite_permission::roleid
                        .is_null()
                        .and(
                            campsite_permission::userid
                            .ne(&auth.actor.did)
                        )
                        .or::<bool, Bool>(
                            !non_self
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|x| campsite_permission_view(x))
        .collect::<Vec<CampsitePermissionView>>();

    Ok(Json(GetCampsitePermissionsOutput { permissions }))
}
