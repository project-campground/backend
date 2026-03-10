use appview_schema::{models::appview::CampsiteRole, schema::appview::{campsite_permission, campsite_role}};
use campground_lexicon::gg::campground::campsite::CampsitePermissionViewDetailed;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{api::gg::campground::permission::update_permission::{UpdatePermissionBody, create_or_modify_permission, ensure_update_permission_good_request}, database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_permission_view_detailed, permissions::{GeneralPermissionConsts, has_full_leveled_perms_from_roles}, roles::ensure_no_higher_role}, realtime::data::ReactiveSubject, xrpc::{
    campsite::{BonfireInfo, CategoryInfo, TentInfo}, error::{Result, XRPCError}
}};

pub async fn update_tent_role_permission(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, role_id: Uuid, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionViewDetailed>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.tent.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !(auth.campsite.owner == auth.actor.did || has_full_leveled_perms_from_roles(&auth.campsite.id, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id.clone()), &all_roles, &auth.member, GeneralPermissionConsts::MANAGE_ROLES, 0).await?) {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        event_subject,
        &auth.actor.did,
        &auth.tent.campsite_id,
        &auth.tent.bonfire_id,
        None,
        Some(auth.tent.id),
        Some(role_id),
        None,
        &inner_body.permissions,
        &campsite_permission::roleid
            .eq(role_id)
            .and(
                campsite_permission::tentid
                .eq(auth.tent.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view_detailed(permission)))
}

pub async fn update_category_role_permission(auth: CategoryInfo<'_>, event_subject: &State<ReactiveSubject>, role_id: Uuid, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionViewDetailed>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.category.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !(auth.campsite.owner == auth.actor.did || has_full_leveled_perms_from_roles(&auth.campsite.id, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &all_roles, &auth.member, GeneralPermissionConsts::MANAGE_ROLES, 0).await?) {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        event_subject,
        &auth.actor.did,
        &auth.category.campsite_id,
        &auth.category.bonfire_id,
        Some(auth.category.id),
        None,
        Some(role_id),
        None,
        &inner_body.permissions,
        &campsite_permission::roleid
            .eq(role_id)
            .and(
                campsite_permission::categoryid
                    .eq(auth.category.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view_detailed(permission)))
}

pub async fn update_bonfire_role_permission(auth: BonfireInfo<'_>, event_subject: &State<ReactiveSubject>, role_id: Uuid, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionViewDetailed>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.bonfire.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !(auth.campsite.owner == auth.actor.did || has_full_leveled_perms_from_roles(&auth.campsite.id, &auth.bonfire.id, None, None, &all_roles, &auth.member, GeneralPermissionConsts::MANAGE_ROLES, 0).await?) {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        event_subject,
        &auth.actor.did,
        &auth.bonfire.campsite_id,
        &auth.bonfire.id,
        None,
        None,
        Some(role_id),
        None,
        &inner_body.permissions,
        &campsite_permission::roleid
            .eq(role_id)
            .and(
                campsite_permission::bonfireid
                    .eq(&auth.bonfire.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view_detailed(permission)))
}
