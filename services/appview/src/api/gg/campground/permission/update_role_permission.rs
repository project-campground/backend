use appview_schema::{models::appview::{CampsitePermission, CampsiteRole}, schema::appview::{campsite_permission, campsite_role}};
use campground_lexicon::gg::campground::campsite::CampsitePermissionView;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, expression::NonAggregate, sql_types::BoolOrNullableBool};
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_permission_view, permissions::{CampsitePermissionConsts, has_full_tent_perms_from_roles}, roles::ensure_no_higher_role}, xrpc::{
    campsite::{BonfireInfo, CategoryInfo, TentInfo}, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdatePermissionBody {
    pub allowed_tent_permissions: i64,
    pub denied_tent_permissions: i64,
    pub allowed_campsite_permissions: i64,
    pub denied_campsite_permissions: i64,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<tent_id>&<role_id>", data = "<body>", rank = 1)]
pub async fn update_tent_role_permission(auth: TentInfo<'_>, role_id: &str, tent_id: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'role_id' query to be a valid UUID".to_string()))
        ?;
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.tent.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id_uuid);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !has_full_tent_perms_from_roles(&auth.campsite.id, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id.clone()), &all_roles, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.tent.campsite_id,
        None,
        None,
        Some(auth.tent.id),
        Some(role_id_uuid),
        None,
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::roleid
            .eq(role_id_uuid)
            .and(
                campsite_permission::tentid
                .eq(auth.tent.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<category_id>&<role_id>", data = "<body>", rank = 2)]
pub async fn update_category_role_permission(auth: CategoryInfo<'_>, role_id: &str, category_id: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'role_id' query to be a valid UUID".to_string()))
        ?;
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.category.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id_uuid);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !has_full_tent_perms_from_roles(&auth.campsite.id, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &all_roles, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.category.campsite_id,
        None,
        Some(auth.category.id),
        None,
        Some(role_id_uuid),
        None,
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::roleid
            .eq(role_id_uuid)
            .and(
                campsite_permission::categoryid
                    .eq(auth.category.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<bonfire_id>&<role_id>", data = "<body>", rank = 3)]
pub async fn update_bonfire_role_permission(auth: BonfireInfo<'_>, role_id: &str, bonfire_id: &str, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionView>> {    
    let inner_body = &body.into_inner();

    ensure_update_permission_good_request(inner_body)?;

    let mut conn = establish_connection().unwrap();
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'role_id' query to be a valid UUID".to_string()))
        ?;
    
    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(&auth.bonfire.campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id_uuid);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    if !has_full_tent_perms_from_roles(&auth.campsite.id, &auth.bonfire.id, None, None, &all_roles, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    
    let given_role = given_role.unwrap();

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;
    
    let permission = &create_or_modify_permission(
        &auth.actor.did,
        &auth.bonfire.campsite_id,
        Some(auth.bonfire.id.clone()),
        None,
        None,
        Some(role_id_uuid),
        None,
        inner_body.allowed_campsite_permissions,
        inner_body.allowed_tent_permissions,
        inner_body.denied_campsite_permissions,
        inner_body.denied_tent_permissions,
        &campsite_permission::roleid
            .eq(role_id_uuid)
            .and(
                campsite_permission::bonfireid
                    .eq(auth.bonfire.id)
            ),
    )?;
    
    Ok(Json(campsite_permission_view(permission)))
}

pub fn ensure_update_permission_good_request(inner_body: &UpdatePermissionBody) -> Result<()> {
    if (inner_body.allowed_campsite_permissions & inner_body.denied_campsite_permissions) | (inner_body.allowed_tent_permissions & inner_body.denied_tent_permissions) != 0 {
        Err(XRPCError::BadRequest("Cannot both allow and deny the same permission".to_string()))
    } else {
        Ok(())
    }
}

pub fn create_or_modify_permission<Predicate: diesel::Expression + diesel::expression::ValidGrouping<()> + diesel::AppearsOnTable<appview_schema::schema::appview::campsite_permission::table> + diesel::query_builder::QueryFragment<diesel::pg::Pg> + diesel::query_builder::QueryId>(
    actor: &String,
    campsite_id: &String,
    bonfire_id: Option<String>,
    category_id: Option<Uuid>,
    tent_id: Option<Uuid>,
    role_id: Option<Uuid>,
    user_id: Option<String>,
    allowed_campsite_permissions: i64,
    allowed_tent_permissions: i64,
    denied_campsite_permissions: i64,
    denied_tent_permissions: i64,
    predicate: &Predicate,
) -> Result<CampsitePermission>
    where 
          Predicate : Clone,
          Predicate : NonAggregate,
          <Predicate as diesel::Expression>::SqlType: BoolOrNullableBool,
        //   <Predicate as ValidGrouping<()>>::IsAggregate: MixedAggregates<diesel::expression::is_aggregate::No>,
{
    let mut conn = establish_connection().unwrap();
    let existing = &campsite_permission::table
        .filter(
            predicate.clone()
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?;

    if existing.len() < 1 {
        let current_date = Utc::now().naive_utc();
        Ok(diesel::insert_into(campsite_permission::table)
            .values(CampsitePermission {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.clone(),
                bonfire_id,
                category_id,
                tent_id,
                role_id,
                user_id,
                created_by: actor.clone(),
                created_at: current_date,
                updated_by: actor.clone(),
                updated_at: current_date,
                allowed_campsite_permissions,
                allowed_tent_permissions,
                denied_campsite_permissions,
                denied_tent_permissions,
            })
            .load::<CampsitePermission>(&mut conn)
            .map_err(handle_select_first_error)?
            .first()
            .unwrap()
            .clone())
    } else {
        Ok(diesel::update(campsite_permission::table)
            .filter(
                predicate
            )
            .set((
                campsite_permission::allowedcampsitepermissions
                    .eq(
                        allowed_campsite_permissions
                    ),
                campsite_permission::deniedcampsitepermissions
                    .eq(
                        denied_campsite_permissions
                    ),
                campsite_permission::allowedtentpermissions
                    .eq(
                        allowed_campsite_permissions
                    ),
                campsite_permission::deniedtentpermissions
                    .eq(
                        denied_tent_permissions
                    )
            ))
            .get_result::<CampsitePermission>(&mut conn)
            .map_err(handle_select_first_error)?)
    }
}
