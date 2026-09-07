use appview_schema::{
    models::appview::{CampsiteMember, CampsiteRole},
    schema::appview::{campsite_member, campsite_role},
};
use campground_lexicon::gg::campground::membership::ModifyMemberRolesOutput;
use diesel::pg::expression::dsl::array_remove;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgArrayExpressionMethods, QueryDsl, RunQueryDsl,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, has_role_perms_or_owner},
        ws::event_next,
    },
    realtime::data::{ReactiveSubject, ReactiveSubjectData},
    views::roles::{CampsiteRoleFlag, ensure_no_higher_role, role_view_basic},
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct RemoveMemberRoleBody {
    member_ids: Vec<String>,
}

#[post(
    "/xrpc/gg.campground.role.removeMemberRoles?<campsite_id>&<role_id>",
    data = "<body>"
)]
pub async fn remove_member_role(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    role_id: &str,
    body: Json<RemoveMemberRoleBody>,
) -> Result<Json<ModifyMemberRolesOutput>> {
    let inner_body = &body.into_inner();
    let member_ids_length = inner_body.member_ids.len();
    if member_ids_length < 1 || member_ids_length > 100 {
        return Err(XRPCError::BadRequest(
            "Expected 'member_ids' property to have an array of length 1 to 100".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id).map_err(|_| {
        XRPCError::BadRequest("Expected 'role_id' query to be a valid UUID".to_string())
    })?;

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
            "Cannot remove default role from a member".to_string(),
        ));
    }

    ensure_no_higher_role(
        auth.campsite.owner == auth.actor.did,
        &mut all_roles.clone(),
        given_role.priority,
        auth.member.roles.clone(),
    )?;

    expect_permission!(has_role_perms_or_owner(
        &auth.campsite,
        &auth.member,
        GeneralPermissionConsts::GIVE_ROLES,
        0
    ));

    let updated_members = diesel::update(campsite_member::table)
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq_any(inner_body.member_ids.clone()))
                .and(campsite_member::roles.contains(vec![given_role.id])),
        )
        .set((
            // Stuff changed
            campsite_member::roles.eq(array_remove(campsite_member::roles, role_id_uuid)),
        ))
        .load::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let member_ids = updated_members
        .iter()
        .map(|x| x.user_id.clone())
        .collect::<Vec<String>>();
    let role_view = role_view_basic(given_role);
    let output = ModifyMemberRolesOutput {
        role: role_view.clone(),
        members: member_ids.clone(),
    };

    event_next(event_subject, "MemberRolesRemoved", &output, |payload| {
        ReactiveSubjectData::MemberRolesModified {
            campsite_id: campsite_id.to_string(),
            actors: member_ids.clone(),
            role_id: given_role.id,
            permissions_are_empty: given_role.general_permissions == 0
                && given_role.content_permissions == 0,
            removed: true,
            binary: payload,
        }
    });

    return Ok(Json(output));
}
