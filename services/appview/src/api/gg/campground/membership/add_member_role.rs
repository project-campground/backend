use appview_schema::{models::appview::{CampsiteMember, CampsiteRole}, schema::appview::{campsite_member, campsite_role}};
use campground_lexicon::gg::campground::membership::ModifyMemberRolesOutput;
use diesel::{BoolExpressionMethods, ExpressionMethods, PgArrayExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;
use diesel::dsl::not;

use crate::{database::establish_connection, expect_permission, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{GeneralPermissionConsts, has_role_perms_or_owner}, roles::ensure_no_higher_role, ws::event_next}, realtime::data::{ReactiveSubject, ReactiveSubjectData}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct AddMemberRoleBody {
    member_ids: Vec<String>,
}

#[post("/xrpc/gg.campground.membership.addMemberRoles?<campsite_id>&<role_id>", data = "<body>")]
pub async fn add_member_role(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, role_id: &str, body: Json<AddMemberRoleBody>) -> Result<Json<ModifyMemberRolesOutput>> {    
    let inner_body = &body.into_inner();
    let member_ids_length = inner_body.member_ids.len();
    if member_ids_length < 1 || member_ids_length > 100 {
        return Err(XRPCError::BadRequest("Expected 'member_ids' property to have an array of length 1 to 100".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'role_id' query to be a valid UUID".to_string()))
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

    ensure_no_higher_role(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), given_role.priority, auth.member.roles.clone())?;

    expect_permission!(has_role_perms_or_owner(&auth.campsite, &auth.member, GeneralPermissionConsts::GIVE_ROLES, 0));
    
    let updated_members = diesel::update(campsite_member::table)
        .filter(
            campsite_member::campsiteid.eq(campsite_id)
                .and(
                    campsite_member::userid
                        .eq_any(inner_body.member_ids.clone())
                )
                .and(
                    not(
                        campsite_member::roles
                            .contains(
                                vec![ given_role.id ]
                            )
                    )
                )
        )
        .set((
            // Stuff changed
            campsite_member::roles
                .eq(
                    campsite_member::roles
                        .concat(vec![role_id_uuid])
                ),
        ))
        .load::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    let member_ids = updated_members.iter().map(|x| x.user_id.clone()).collect::<Vec<String>>();
    let role_view = campsite_role_view_basic(given_role);

    event_next(event_subject, "MemberRolesAdded", ModifyMemberRolesOutput { role: role_view.clone(), members: member_ids.clone() }, |payload|
        ReactiveSubjectData::MemberRolesModified {
            campsite_id: campsite_id.to_string(),
            actors: member_ids.clone(),
            role_id: given_role.id,
            permissions_are_empty: given_role.general_permissions == 0 && given_role.content_permissions == 0,
            removed: false,
            binary: payload
    });

    return Ok(Json(ModifyMemberRolesOutput { role: role_view, members: member_ids }));
}