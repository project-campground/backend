use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::{
    PermissionViewDetailed, PermissionsStateDictionary,
};
use chrono::Utc;
use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, expression::NonAggregate,
    sql_types::BoolOrNullableBool,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::gg::campground::permission::{
        update_role_permission::{
            update_bonfire_role_permission, update_category_role_permission,
            update_tent_role_permission,
        },
        update_user_permission::{
            update_bonfire_user_permission, update_category_user_permission,
            update_tent_user_permission,
        },
    },
    database::establish_connection,
    helpers::{api::handle_select_first_error, ws::event_next},
    realtime::data::{ReactiveSubject, ReactiveSubjectData},
    views::permissions::permission_view_detailed,
    xrpc::{campsite::OneOfInfo, error::XRPCError},
};

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
#[post(
    "/xrpc/gg.campground.permission.updatePermission?<query..>",
    data = "<body>",
    rank = 1
)]
pub async fn update_permission(
    auth: OneOfInfo<'_>,
    query: UpdatePermissionsQuery,
    event_subject: &State<ReactiveSubject>,
    body: Json<UpdatePermissionBody>,
) -> Result<Json<PermissionViewDetailed>, XRPCError> {
    match (auth, query.role_id, query.actor) {
        // Roles
        (OneOfInfo::Tent(tent_auth), Some(role_id), None) => {
            update_tent_role_permission(tent_auth, event_subject, role_id, body).await
        }
        (OneOfInfo::Category(category_auth), Some(role_id), None) => {
            update_category_role_permission(category_auth, event_subject, role_id, body).await
        }
        (OneOfInfo::Bonfire(bonfire_auth), Some(role_id), None) => {
            update_bonfire_role_permission(bonfire_auth, event_subject, role_id, body).await
        }
        // Users
        (OneOfInfo::Tent(tent_auth), None, Some(actor)) => {
            update_tent_user_permission(tent_auth, event_subject, &actor, body).await
        }
        (OneOfInfo::Category(category_auth), None, Some(actor)) => {
            update_category_user_permission(category_auth, event_subject, &actor, body).await
        }
        (OneOfInfo::Bonfire(bonfire_auth), None, Some(actor)) => {
            update_bonfire_user_permission(bonfire_auth, event_subject, &actor, body).await
        }
        _ => Err(XRPCError::BadRequest(
            "Expected only 'role_id' or 'actor' query parameters, not none or both.".to_string(),
        )),
    }
}

pub fn ensure_update_permission_good_request(
    inner_body: &UpdatePermissionBody,
) -> Result<(), XRPCError> {
    if (inner_body.permissions.allowed.general & inner_body.permissions.denied.general)
        | (inner_body.permissions.allowed.content & inner_body.permissions.denied.content)
        != 0
    {
        Err(XRPCError::BadRequest(
            "Cannot both allow and deny the same permission".to_string(),
        ))
    } else {
        Ok(())
    }
}

pub fn create_or_modify_permission<
    Predicate: diesel::Expression
        + diesel::expression::ValidGrouping<()>
        + diesel::AppearsOnTable<appview_schema::schema::appview::campsite_permission::table>
        + diesel::query_builder::QueryFragment<diesel::pg::Pg>
        + diesel::query_builder::QueryId,
>(
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
) -> Result<PermissionViewDetailed, XRPCError>
where
    Predicate: Clone,
    Predicate: NonAggregate,
    <Predicate as diesel::Expression>::SqlType: BoolOrNullableBool,
{
    let mut conn = establish_connection().unwrap();
    let existing = &campsite_permission::table
        .filter(predicate.clone())
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?;

    let permission = (if existing.len() < 1 {
        insert_permission_if_not_empty(
            actor,
            campsite_id,
            bonfire_id,
            category_id,
            tent_id,
            role_id,
            user_id,
            permissions,
        )
    } else {
        update_or_delete_role_permission(permissions, existing.first().unwrap().clone(), predicate)
    })?;

    let permission_view = permission_view_detailed(&permission);

    event_next(
        event_subject,
        "PermissionUpdated",
        &permission_view,
        |binary| ReactiveSubjectData::CampsitePermissionUpdated {
            campsite_id: permission.campsite_id.clone(),
            bonfire_id: permission.bonfire_id.clone(),
            category_id: permission.category_id.clone(),
            tent_id: permission.tent_id.clone(),
            user_id: permission.user_id.clone(),
            role_id: permission.role_id.clone(),
            binary,
        },
    );
    Ok(permission_view)
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

    if permissions.allowed.general
        | permissions.allowed.content
        | permissions.denied.general
        | permissions.denied.content
        == 0
    {
        return Err(XRPCError::BadRequest(
            "Expected at least one denied or allowed permission".to_string(),
        ));
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
            allowed_general_permissions: permissions.allowed.general as i64,
            allowed_content_permissions: permissions.allowed.content as i64,
            denied_general_permissions: permissions.denied.general as i64,
            denied_content_permissions: permissions.denied.content as i64,
        })
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?
        .first()
        .unwrap()
        .clone())
}

fn update_or_delete_role_permission<
    Predicate: diesel::Expression
        + diesel::expression::ValidGrouping<()>
        + diesel::AppearsOnTable<appview_schema::schema::appview::campsite_permission::table>
        + diesel::query_builder::QueryFragment<diesel::pg::Pg>
        + diesel::query_builder::QueryId,
>(
    permissions: &PermissionsStateDictionary,
    existing: CampsitePermission,
    predicate: &Predicate,
) -> Result<CampsitePermission, XRPCError>
where
    Predicate: Clone,
    Predicate: NonAggregate,
    <Predicate as diesel::Expression>::SqlType: BoolOrNullableBool,
{
    let mut conn = establish_connection().unwrap();
    if permissions.allowed.general
        | permissions.allowed.content
        | permissions.denied.general
        | permissions.denied.content
        == 0
    {
        diesel::delete(campsite_permission::table)
            .filter(predicate.clone())
            .execute(&mut conn)
            .map_err(handle_select_first_error)?;
        return Ok(existing);
    }

    Ok(diesel::update(campsite_permission::table)
        .filter(predicate.clone())
        .set((
            campsite_permission::allowedgeneralpermissions.eq(permissions.allowed.general as i64),
            campsite_permission::deniedgeneralpermissions.eq(permissions.denied.general as i64),
            campsite_permission::allowedcontentpermissions.eq(permissions.allowed.content as i64),
            campsite_permission::deniedcontentpermissions.eq(permissions.denied.content as i64),
        ))
        .get_result::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)?)
}
