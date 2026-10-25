use appview_schema::{
    models::appview::CampsiteRole,
    schema::appview::{self, campsite_role},
};
use campground_lexicon::gg::campground::role::RoleViewBasic;
use chrono::Utc;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{DbConnection, campsites::get_roles_from_db, establish_connection},
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
        ws::event_next_campsite,
    },
    make_room_for_items,
    realtime::data::ReactiveSubject,
    views::roles::role_view_basic,
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveRoleBody {
    position: i32,
}

#[allow(unused_variables)]
#[post(
    "/xrpc/gg.campground.role.moveRole?<campsite_id>&<role_id>",
    data = "<body>"
)]
pub async fn move_role(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    role_id: &str,
    body: Json<MoveRoleBody>,
) -> Result<Json<RoleViewBasic>> {
    let MoveRoleBody { position } = &body.into_inner();

    let mut conn = establish_connection().unwrap();

    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))?;

    // Make sure the role exists
    let roles = get_roles_from_db(campsite_id)?;

    let given_role = roles.iter().find(|x| x.id == role_id_uuid);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }
    let given_role = given_role.unwrap();

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::MANAGE_ROLES,
        0
    ));

    let actor_max_priority = roles
        .iter()
        .filter(|x| auth.member.roles.contains(&Some(x.id)))
        .min_by(|x, y| x.priority.cmp(&y.priority))
        .map_or(i32::MAX, |role| role.priority);
    let role_pos = given_role.priority;

    if auth.campsite.owner != auth.member.user_id && actor_max_priority <= *position {
        return Err(XRPCError::Forbidden(
            "The given position is lower than actor's minimum role position".to_string(),
        ));
    }

    if auth.campsite.owner != auth.member.user_id && actor_max_priority <= role_pos {
        return Err(XRPCError::Forbidden(
            "The supplied role's position is lower than actor's minimum role position".to_string(),
        ));
    }

    let updated_roles = conn
        .transaction(|conn| {
            make_room_for_role(conn, campsite_id, *position)?;

            let current_date = Utc::now().naive_utc();

            diesel::update(campsite_role::table)
                .filter(campsite_role::id.eq(role_id_uuid))
                .set((
                    // Mandatory
                    campsite_role::updatedby.eq(&auth.actor.did),
                    campsite_role::updatedat.eq(current_date),
                    // Stuff changed
                    campsite_role::priority.eq(*position),
                ))
                .load::<CampsiteRole>(conn)
        })
        .map_err(handle_select_first_error)?;

    let updated_role = updated_roles.first().unwrap();

    let role_view = role_view_basic(updated_role);

    event_next_campsite(event_subject, &auth.campsite.id, 0, "RoleMoved", &role_view);

    return Ok(Json(role_view));
}

fn make_room_for_role(
    conn: &mut DbConnection,
    campsite_id: &str,
    position: i32,
) -> Result<Vec<(Uuid, i32)>, diesel::result::Error> {
    use diesel::BoolExpressionMethods;
    use diesel::QueryDsl;
    use diesel::sql_types::BigInt;
    use std::ops::Add;

    make_room_for_items!(
        crate::schema::appview::campsite_role::table,
        crate::schema::appview::campsite_role::id,
        crate::schema::appview::campsite_role::priority,
        appview::campsite_role::campsiteid
            .eq(campsite_id);
        conn,
        position
    );
}
