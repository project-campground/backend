use appview_schema::{
    models::appview::{CampsiteMember, CampsiteRole},
    schema::appview::{campsite_member, campsite_permission, campsite_role},
};
use campground_lexicon::gg::campground::role::RoleViewBasic;
use diesel::{Connection, ExpressionMethods, PgArrayExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
        ws::event_next_campsite,
    },
    realtime::data::ReactiveSubject,
    views::roles::{CampsiteRoleFlag, ensure_no_higher_role, role_view_basic},
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/gg.campground.role.deleteRole?<campsite_id>&<role_id>")]
pub async fn delete_role(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    role_id: &str,
) -> Result<Json<RoleViewBasic>> {
    let mut conn = establish_connection().unwrap();

    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))?;

    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let given_role = all_roles.iter().find(|x| x.id == role_id_uuid);
    if given_role.is_none() {
        return Err(XRPCError::NotFound);
    }

    let given_role = given_role.unwrap();

    if given_role.flags & CampsiteRoleFlag::DEFAULT_ROLE != 0 {
        return Err(XRPCError::Forbidden(
            "Cannot delete default role".to_string(),
        ));
    }

    ensure_no_higher_role(
        auth.actor.did == auth.campsite.owner,
        &mut all_roles.clone(),
        given_role.priority,
        auth.member.roles.clone(),
    )?;

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::MANAGE_ROLES,
        0
    ));

    conn.transaction(|conn| {
        diesel::delete(campsite_role::table)
            .filter(campsite_role::id.eq(given_role.id))
            .execute(conn)?;
        diesel::delete(campsite_permission::table)
            .filter(campsite_permission::roleid.eq(given_role.id))
            .execute(conn)?;
        diesel::update(campsite_member::table)
            .filter(campsite_member::roles.contains(vec![given_role.id]))
            .set(campsite_member::roles.eq(diesel::dsl::array_remove(
                campsite_member::roles,
                given_role.id,
            )))
            .load::<CampsiteMember>(conn)
    })
    .map_err(handle_all_db_errors)?;

    let view = role_view_basic(given_role);
    event_next_campsite(event_subject, &auth.campsite.id, 0, "RoleDeleted", &view);

    return Ok(Json(view));
}
