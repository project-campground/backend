use appview_schema::{models::appview::CampsiteRole, schema::appview::{campsite_permission, campsite_role}};
use campground_lexicon::gg::campground::campsite::CampsiteRoleViewBasic;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, roles::{CampsiteRoleFlag, ensure_no_higher_role}, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.campsite.deleteRole?<campsite_id>&<role_id>")]
pub async fn delete_role(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, role_id: &str) -> Result<Json<CampsiteRoleViewBasic>> {    
    let mut conn = establish_connection().unwrap();

    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;

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
        return Err(XRPCError::Forbidden("Cannot delete default role".to_string()));
    }

    ensure_no_higher_role(auth.actor.did == auth.campsite.owner, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;

    if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::MANAGE_ROLES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    diesel::delete(campsite_role::table)
        .filter(
            campsite_role::id.eq(given_role.id)
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;
    diesel::delete(campsite_permission::table)
        .filter(
            campsite_permission::roleid
                .eq(given_role.id)
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next(event_subject, &auth.campsite.id, "RoleDeleted", campsite_role_view_basic(given_role));

    return Ok(Json(campsite_role_view_basic(given_role)));
}