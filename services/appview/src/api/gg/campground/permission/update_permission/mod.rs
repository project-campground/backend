use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::PermissionViewBasic;
use chrono::Utc;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgExpressionMethods, RunQueryDsl,
    upsert::on_constraint,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::gg::campground::permission::update_permission::database::fetch_role_priorities,
    database::{DbConnection, campsites::get_campsite_member, establish_connection},
    helpers::{
        api::handle_all_db_errors, permissions::state::INVALID_PERMISSION_FLAG_MASK, ws::event_next,
    },
    realtime::data::{ReactiveSubject, ReactiveSubjectData},
    views::permissions::permission_view_basic,
    xrpc::{campsite::OneOfInfo, error::XRPCError},
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdatePermissionBody {
    pub permissions: u64,
}

#[derive(FromForm)]
pub struct UpdatePermissionsQuery {
    #[allow(dead_code)]
    tent_id: Option<Uuid>,
    #[allow(dead_code)]
    category_id: Option<Uuid>,
    #[allow(dead_code)]
    bonfire_id: Option<String>,
    actor: Option<String>,
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
) -> Result<Json<PermissionViewBasic>, XRPCError> {
    let UpdatePermissionBody { permissions } = body.into_inner();

    if has_invalid_flag(&permissions) {
        return Err(XRPCError::BadRequest(
            "Unexpected 0b01 to exist as a permission flag value. Only 0b10, 0b11 and 0b00 flag values are permitted".to_string(),
        ));
    }

    match (query.role_id, query.actor) {
        (Some(role_id), None) => {
            let target_roles = vec![role_id];

            let mut conn = establish_connection().unwrap();
            conn.transaction(move |conn| {
                update_permission_in_database(
                    conn,
                    event_subject,
                    auth,
                    target_roles,
                    &query.category_id,
                    &query.tent_id,
                    Some(&role_id),
                    None,
                    permissions,
                )
            })
        }
        (None, Some(actor)) => {
            let target_actor = get_campsite_member(&auth.campsite(), &actor)?;
            let target_roles = target_actor
                .roles
                .iter()
                .filter_map(|&x| x)
                .collect::<Vec<Uuid>>();

            let mut conn = establish_connection().unwrap();
            conn.transaction(move |conn| {
                update_permission_in_database(
                    conn,
                    event_subject,
                    auth,
                    target_roles,
                    &query.category_id,
                    &query.tent_id,
                    None,
                    Some(&actor),
                    permissions,
                )
            })
        }
        _ => Err(XRPCError::BadRequest(
            "Expected only 'role_id' or 'actor' query parameters, not none or both.".to_string(),
        )),
    }
}

// Hoping this bullshit works as intended and I am not dumb
pub fn has_invalid_flag(permission: &u64) -> bool {
    let only_last_flag_digit = permission & INVALID_PERMISSION_FLAG_MASK;
    let first_digit_as_inverted_last = !(permission >> 1);

    (only_last_flag_digit & first_digit_as_inverted_last) != 0u64
}

pub fn get_priority_from_role_list(role_priorities: &Vec<(Uuid, i32)>, roles: Vec<Uuid>) -> i32 {
    role_priorities
        .iter()
        .filter(|x| roles.contains(&x.0))
        .map(|x| x.1)
        .min()
        .unwrap()
}

pub fn update_permission_in_database<'a>(
    conn: &mut DbConnection,
    event_subject: &State<ReactiveSubject>,
    auth: OneOfInfo<'a>,
    target_roles: Vec<Uuid>,
    category_id: &Option<Uuid>,
    tent_id: &Option<Uuid>,
    role_id: Option<&Uuid>,
    user_id: Option<&String>,
    permissions: u64,
) -> Result<Json<PermissionViewBasic>, XRPCError> {
    let executor_roles = auth
        .member_roles()
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<_>>();
    let priorities = fetch_role_priorities(conn, &auth.campsite(), &executor_roles, &target_roles)?;
    let target_priorities = get_priority_from_role_list(&priorities, target_roles);
    let executor_priorities = get_priority_from_role_list(&priorities, executor_roles);

    // Smaller priority, the higher in the role list
    if executor_priorities > target_priorities {
        return Err(XRPCError::Forbidden(
            "Cannot update permissions of higher role or of member who has higher roles"
                .to_string(),
        ));
    }

    let bonfire_id = &auth.bonfire_id();
    let campsite_id = &auth.campsite();
    let executor = &auth.actor_did();

    let modified_permissions = conn
        .transaction(move |conn| {
            if permissions == 0 {
                delete_permission(
                    conn,
                    campsite_id.clone(),
                    bonfire_id.clone(),
                    category_id.clone(),
                    tent_id.clone(),
                    role_id.cloned(),
                    user_id.cloned(),
                )
            } else {
                create_or_update_permission(
                    conn,
                    executor.clone(),
                    campsite_id.clone(),
                    bonfire_id.clone(),
                    category_id.clone(),
                    tent_id.clone(),
                    role_id.cloned(),
                    user_id.cloned(),
                    permissions,
                )
            }
        })
        .map_err(handle_all_db_errors)?;

    let permission = modified_permissions.first().map_or_else(
        || PermissionViewBasic {
            permissions,
            bonfire_id: bonfire_id.clone(),
            category_id: category_id.clone(),
            tent_id: tent_id.clone(),
            user_id: user_id.cloned(),
            role_id: role_id.cloned(),
        },
        |permission| permission_view_basic(permission),
    );

    event_next(event_subject, "PermissionUpdated", &permission, |binary| {
        ReactiveSubjectData::CampsitePermissionUpdated {
            campsite_id: campsite_id.clone(),
            bonfire_id: bonfire_id.clone(),
            category_id: *category_id,
            tent_id: *tent_id,
            user_id: user_id.cloned(),
            role_id: role_id.cloned(),
            binary,
        }
    });

    Ok(Json(permission))
}

pub fn delete_permission(
    conn: &mut DbConnection,
    campsite_id: String,
    bonfire_id: String,
    category_id: Option<Uuid>,
    tent_id: Option<Uuid>,
    role_id: Option<Uuid>,
    user_id: Option<String>,
) -> Result<Vec<CampsitePermission>, diesel::result::Error> {
    diesel::delete(campsite_permission::table)
        .filter(
            campsite_permission::campsiteid.eq(campsite_id).and(
                campsite_permission::bonfireid
                    .eq(bonfire_id)
                    .and(campsite_permission::categoryid.is_not_distinct_from(category_id))
                    .and(campsite_permission::tentid.is_not_distinct_from(tent_id))
                    .and(campsite_permission::roleid.is_not_distinct_from(role_id))
                    .and(campsite_permission::userid.is_not_distinct_from(user_id)),
            ),
        )
        .returning(campsite_permission::all_columns)
        .load::<CampsitePermission>(conn)
}

pub fn create_or_update_permission(
    conn: &mut DbConnection,
    created_by: String,
    campsite_id: String,
    bonfire_id: String,
    category_id: Option<Uuid>,
    tent_id: Option<Uuid>,
    role_id: Option<Uuid>,
    user_id: Option<String>,
    permissions: u64,
) -> Result<Vec<CampsitePermission>, diesel::result::Error> {
    let current_date = Utc::now().naive_utc();

    diesel::insert_into(campsite_permission::table)
        .values(CampsitePermission {
            id: uuid::Uuid::new_v4(),
            campsite_id,
            role_id,
            user_id,
            category_id,
            tent_id,
            bonfire_id,
            permissions: permissions as i64,
            created_by: created_by.clone(),
            created_at: current_date,
            updated_by: created_by.clone(),
            updated_at: current_date,
        })
        .on_conflict(on_constraint("campsite_permission_unique_keys"))
        .do_update()
        .set((
            campsite_permission::permissions.eq(permissions as i64),
            campsite_permission::updatedby.eq(created_by.clone()),
            campsite_permission::updatedat.eq(current_date),
        ))
        .returning(campsite_permission::all_columns)
        .load::<CampsitePermission>(conn)
}

pub mod database;
