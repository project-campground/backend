use std::collections::HashMap;

use appview_schema::{models::appview::CampsiteRole, schema::appview::campsite_role};
use campground_lexicon::gg::campground::campsite::{CampsiteRoleViewBasic, CampsiteRolesMovedOutput, GetCampsiteRolesOutput};
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl, dsl::sql, sql_types::{Array, Integer, Text}};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{campsites::get_roles_from_db, establish_connection}, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{GeneralPermissionConsts, has_role_perms_or_owner}, ws::event_next_campsite}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveRoleBody {
    roles_by_priority: HashMap<Uuid, i32>,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.campsite.moveRoles?<campsite_id>", data = "<body>")]
pub async fn move_roles(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, body: Json<MoveRoleBody>) -> Result<Json<GetCampsiteRolesOutput>> {    
    let MoveRoleBody { roles_by_priority } = &body.into_inner();

    if roles_by_priority.len() < 1 {
        return Err(XRPCError::BadRequest("Expected at least one role provided".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let roles = &get_roles_from_db(&auth.campsite.id)?;
    let role_ids = roles.iter().map(|x| x.id).collect::<Vec<Uuid>>();

    let role_not_found = roles_by_priority.iter().find(|x| !role_ids.contains(x.0));
    if role_not_found.is_some() {
        return Err(XRPCError::BadRequest(format!("Did not find '{}' role in this campsite", role_not_found.unwrap().0)));
    }
    
    if !has_role_perms_or_owner(&auth.campsite, &auth.member, GeneralPermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let actor_max_priority = roles
        .iter()
        .filter(|x| auth.member.roles.contains(&Some(x.id)))
        .max_by(|x, y| x.priority.cmp(&y.priority))
        .unwrap()
        .priority;

    let max_given_priority = roles_by_priority.values().max().unwrap();

    if auth.campsite.owner != auth.member.user_id && actor_max_priority <= *max_given_priority {
        return Err(XRPCError::Forbidden("One of role supplied priorities is lower than actor's max role priority".to_string()));
    }

    let supplied_role_ids = roles_by_priority.keys();
    let supplied_roles_max_priority = roles
        .iter()
        .filter(|x| supplied_role_ids.clone().find(|y| **y == x.id).is_some())
        .map(|x| x.priority)
        .max()
        .unwrap();

    if auth.campsite.owner != auth.member.user_id && actor_max_priority <= supplied_roles_max_priority {
        return Err(XRPCError::Forbidden("One of the request's roles has lower priority than actor's max role priority".to_string()));
    }

    let current_date = Utc::now().naive_utc();
    let array_role_priorities = roles_by_priority
        .iter()
        .map(|x| format!("{}:{}", x.0, x.1))
        .collect::<Vec<String>>();

    let updated_roles = diesel::update(campsite_role::table)
        .filter(
            campsite_role::id.eq_any(supplied_role_ids)
        )
        .set((
            // Mandatory
            campsite_role::updatedby
                .eq(&auth.actor.did),
            campsite_role::updatedat
                .eq(current_date),
            // Stuff changed
            campsite_role::priority
                .eq(
                    sql::<Integer>(
                        "(\
                        WITH a as (SELECT unnest("
                    )
                    .bind::<Array<Text>, _>(array_role_priorities)
                    .sql(
                        ") AS map) \
                        SELECT (SPLIT_PART(map, ':', 2)::Integer) AS column1 \
                        FROM a \
                        WHERE map LIKE (id || ':%')\
                        )"
                    )
                ),
        ))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(campsite_role_view_basic)
        .collect::<Vec<CampsiteRoleViewBasic>>();

    event_next_campsite(event_subject, &auth.campsite.id, 0, "RolesMoved", CampsiteRolesMovedOutput { roles_by_priority: roles_by_priority.clone(), });

    return Ok(Json(GetCampsiteRolesOutput { roles: updated_roles }));
}
