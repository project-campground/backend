use appview_schema::{models::appview::CampsiteMember, schema::appview::{campsite_member, campsite_permission}};
use campground_lexicon::gg::campground::campsite::CampsitePermissionView;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{api::gg::campground::permission::update_role_permission::{UpdatePermissionBody, create_or_modify_permission, ensure_update_permission_good_request}, database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_permission_view, permissions::{CampsitePermissionConsts, has_full_tent_perms}}, xrpc::{
    campsite::{BonfireInfo, CategoryInfo, TentInfo}, error::{Result, XRPCError}
}};

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<tent_id>&<actor>", data = "<body>", rank = 4)]
pub async fn update_tent_user_permission(auth: TentInfo<'_>, tent_id: &str, actor: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();

    if !has_full_tent_perms(&auth.campsite.id, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id.clone()), &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    campsite_member::table
        .filter(
            campsite_member::userid
                .eq(actor)
                .and(
                    campsite_member::campsiteid
                        .eq(&auth.campsite.id)
                )
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.tent.campsite_id,
        None,
        None,
        Some(auth.tent.id),
        None,
        Some(actor.to_string()),
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::userid
            .eq(actor)
            .and(
                campsite_permission::tentid
                    .eq(auth.tent.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<category_id>&<actor>", data = "<body>", rank = 5)]
pub async fn update_category_user_permission(auth: CategoryInfo<'_>, category_id: &str, actor: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();

    if !has_full_tent_perms(&auth.campsite.id, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    campsite_member::table
        .filter(
            campsite_member::userid
                .eq(actor)
                .and(
                    campsite_member::campsiteid
                        .eq(&auth.campsite.id)
                )
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.category.campsite_id,
        None,
        Some(auth.category.id),
        None,
        None,
        Some(actor.to_string()),
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::userid
            .eq(actor)
            .and(
                campsite_permission::categoryid
                    .eq(auth.category.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<bonfire_id>&<actor>", data = "<body>", rank = 6)]
pub async fn update_bonfire_user_permission(auth: BonfireInfo<'_>, bonfire_id: &str, actor: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();

    if !has_full_tent_perms(&auth.campsite.id, &bonfire_id, None, None, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    campsite_member::table
        .filter(
            campsite_member::userid
                .eq(actor)
                .and(
                    campsite_member::campsiteid
                        .eq(&auth.campsite.id)
                )
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.bonfire.campsite_id,
        Some(auth.bonfire.id.clone()),
        None,
        None,
        None,
        Some(actor.to_string()),
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::userid
            .eq(actor)
            .and(
                campsite_permission::bonfireid
                    .eq(auth.bonfire.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}
