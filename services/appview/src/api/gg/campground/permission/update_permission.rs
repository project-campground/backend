use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::{campsite::CampsitePermissionViewDetailed, permission::PermissionsStateDictionary};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, expression::NonAggregate, sql_types::BoolOrNullableBool};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api::gg::campground::permission::{update_role_permission::{update_bonfire_role_permission, update_category_role_permission, update_tent_role_permission}, update_user_permission::{update_bonfire_user_permission, update_category_user_permission, update_tent_user_permission}}, database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_permission_view_detailed, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{campsite::OneOfInfo, error::XRPCError}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdatePermissionBody {
    pub permissions: PermissionsStateDictionary,
}

#[derive(FromForm)]
pub struct UpdatePermissionsQuery {
    #[allow(dead_code)]
    tent_id: Option<Uuid>,
    #[allow(dead_code)]
    category_id: Option<Uuid>,
    #[allow(dead_code)]
    bonfire_id: Option<String>,
    #[allow(dead_code)]
    actor: Option<String>,
    #[allow(dead_code)]
    role_id: Option<Uuid>,
}


#[allow(unused_variables)]
#[post("/xrpc/gg.campground.permission.updatePermission?<query..>", data = "<body>", rank = 1)]
pub async fn update_permission(auth: OneOfInfo<'_>, query: UpdatePermissionsQuery, event_subject: &State<ReactiveSubject>, body: Json<UpdatePermissionBody>) -> Result<Json<CampsitePermissionViewDetailed>, XRPCError> {
    match (auth, query.role_id, query.actor) {
        // Roles
        (OneOfInfo::Tent(tent_auth), Some(role_id), None) =>
            update_tent_role_permission(tent_auth, event_subject, role_id, body).await,
        (OneOfInfo::Category(category_auth), Some(role_id), None) =>
            update_category_role_permission(category_auth, event_subject, role_id, body).await,
        (OneOfInfo::Bonfire(bonfire_auth), Some(role_id), None) =>
            update_bonfire_role_permission(bonfire_auth, event_subject, role_id, body).await,
        // Users
        (OneOfInfo::Tent(tent_auth), None, Some(actor)) =>
            update_tent_user_permission(tent_auth, event_subject, &actor, body).await,
        (OneOfInfo::Category(category_auth), None, Some(actor)) =>
            update_category_user_permission(category_auth, event_subject, &actor, body).await,
        (OneOfInfo::Bonfire(bonfire_auth), None, Some(actor)) =>
            update_bonfire_user_permission(bonfire_auth, event_subject, &actor, body).await,
        _ => Err(XRPCError::BadRequest("Expected only 'role_id' or 'actor' query parameters, not none or both.".to_string())),
    }
}

pub fn ensure_update_permission_good_request(inner_body: &UpdatePermissionBody) -> Result<(), XRPCError> {
    if (inner_body.permissions.allowed.campsite & inner_body.permissions.denied.campsite) | (inner_body.permissions.allowed.tent & inner_body.permissions.denied.tent) != 0 {
        Err(XRPCError::BadRequest("Cannot both allow and deny the same permission".to_string()))
    } else {
        Ok(())
    }
}

pub fn create_or_modify_permission<Predicate: diesel::Expression + diesel::expression::ValidGrouping<()> + diesel::AppearsOnTable<appview_schema::schema::appview::campsite_permission::table> + diesel::query_builder::QueryFragment<diesel::pg::Pg> + diesel::query_builder::QueryId>(
    event_subject: &State<ReactiveSubject>,
    actor: &String,
    campsite_id: &String,
    bonfire_id: &String,
    category_id: Option<Uuid>,
    tent_id: Option<Uuid>,
    role_id: Option<Uuid>,
    user_id: Option<String>,
    permissions: &PermissionsStateDictionary,
    predicate: &Predicate,
) -> Result<CampsitePermission, XRPCError>
    where 
        Predicate : Clone,
        Predicate : NonAggregate,
        <Predicate as diesel::Expression>::SqlType: BoolOrNullableBool,
{
    let mut conn = establish_connection().unwrap();
    let existing = &campsite_permission::table
        .filter(
            predicate.clone()
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let permission = (if existing.len() < 1 {
        insert_permission_if_not_empty(actor, campsite_id, bonfire_id, category_id, tent_id, role_id, user_id, permissions)
    } else {
        update_or_delete_role_permission(permissions, existing.first().unwrap().clone(), predicate)
    })?;

    event_next(event_subject, campsite_id, "PermissionUpdated", campsite_permission_view_detailed(&permission));
    Ok(permission)
}

fn insert_permission_if_not_empty(
    actor: &String,
    campsite_id: &String,
    bonfire_id: &String,
    category_id: Option<Uuid>,
    tent_id: Option<Uuid>,
    role_id: Option<Uuid>,
    user_id: Option<String>,
    permissions: &PermissionsStateDictionary,
) -> Result<CampsitePermission, XRPCError> {
    let current_date = Utc::now().naive_utc();
    let mut conn = establish_connection().unwrap();

    if permissions.allowed.campsite | permissions.allowed.tent | permissions.denied.campsite | permissions.denied.tent == 0 {
        return Err(XRPCError::BadRequest("Expected at least one denied or allowed permission".to_string()));
    }

    Ok(diesel::insert_into(campsite_permission::table)
        .values(CampsitePermission {
            id: Uuid::new_v4(),
            campsite_id: campsite_id.clone(),
            bonfire_id: bonfire_id.clone(),
            category_id,
            tent_id,
            role_id,
            user_id,
            created_by: actor.clone(),
            created_at: current_date,
            updated_by: actor.clone(),
            updated_at: current_date,
            allowed_campsite_permissions: permissions.allowed.campsite,
            allowed_tent_permissions: permissions.allowed.tent,
            denied_campsite_permissions: permissions.denied.campsite,
            denied_tent_permissions: permissions.denied.tent,
        })
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?
        .first()
        .unwrap()
        .clone())
}

fn update_or_delete_role_permission<Predicate: diesel::Expression + diesel::expression::ValidGrouping<()> + diesel::AppearsOnTable<appview_schema::schema::appview::campsite_permission::table> + diesel::query_builder::QueryFragment<diesel::pg::Pg> + diesel::query_builder::QueryId>(
    permissions: &PermissionsStateDictionary,
    existing: CampsitePermission,
    predicate: &Predicate,
) -> Result<CampsitePermission, XRPCError>
    where
        Predicate : Clone,
        Predicate : NonAggregate,
        <Predicate as diesel::Expression>::SqlType: BoolOrNullableBool,
{
    let mut conn = establish_connection().unwrap();
    if permissions.allowed.campsite | permissions.allowed.tent | permissions.denied.campsite | permissions.denied.tent == 0 {
        diesel::delete(
            campsite_permission::table
        )
            .filter(
                predicate.clone(),
            )
            .execute(&mut conn)
            .map_err(handle_select_first_error)?;
        return Ok(existing);
    }

    Ok(diesel::update(campsite_permission::table)
        .filter(
            predicate.clone(),
        )
        .set((
            campsite_permission::allowedcampsitepermissions
                .eq(
                    permissions.allowed.campsite
                ),
            campsite_permission::deniedcampsitepermissions
                .eq(
                    permissions.denied.campsite
                ),
            campsite_permission::allowedtentpermissions
                .eq(
                        permissions.allowed.tent
                    ),
            campsite_permission::deniedtentpermissions
                .eq(
                    permissions.denied.tent
                )
        ))
        .get_result::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?)

}