use appview_schema::{models::appview::CampsiteRole, schema::appview::campsite_role};
use campground_lexicon::gg::campground::role::{RoleMotion, RoleViewBasic};
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::gg::campground::role::create_role::ensure_user_has_manage_role_permission,
    database::{campsites::get_roles_from_db, establish_connection},
    helpers::{api::handle_select_first_error, ws::event_next_campsite},
    realtime::data::ReactiveSubject,
    views::roles::{ensure_no_higher_role, from_role_motion, role_view_basic},
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateRoleBody {
    name: Option<String>,
    motion: Option<RoleMotion>,
    colors: Option<Vec<u32>>,
    raised: Option<bool>,
    pingable: Option<bool>,
    permissions: u32,
}

#[allow(unused_variables)]
#[post(
    "/xrpc/gg.campground.role.updateRole?<campsite_id>&<role_id>",
    data = "<body>"
)]
pub async fn update_role(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    role_id: &str,
    body: Json<UpdateRoleBody>,
) -> Result<Json<RoleViewBasic>> {
    let UpdateRoleBody {
        name,
        colors,
        motion,
        raised,
        pingable,
        permissions,
    } = &body.into_inner();
    if name.clone().map_or(false, |x| x.len() == 0 || x.len() > 64) {
        return Err(XRPCError::BadRequest(
            "Expected 'name' property to have a string of length 1 to 64 characters".to_string(),
        ));
    } else if colors.clone().map_or(false, |colors| colors.len() > 5) {
        return Err(XRPCError::BadRequest(
            "Expected 'colors' property to be an array with max length of 5".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))?;

    let roles = &get_roles_from_db(&auth.campsite.id)?;
    let role = roles
        .iter()
        .find(|x| x.id == role_id_uuid)
        .ok_or(XRPCError::NotFound)?;

    ensure_user_has_manage_role_permission(&auth.campsite, &auth.member, &roles, permissions)?;

    ensure_no_higher_role(
        auth.member.user_id == auth.campsite.owner,
        &mut roles.clone(),
        role.priority,
        auth.member.roles,
    )?;

    let current_date = Utc::now().naive_utc();

    let updated_role = &diesel::update(campsite_role::table)
        .filter(campsite_role::id.eq(role.id))
        .set((
            // Mandatory
            campsite_role::updatedby.eq(&auth.actor.did),
            campsite_role::updatedat.eq(current_date),
            // Stuff changed
            campsite_role::name.eq(name.clone().unwrap_or(role.name.clone())),
            campsite_role::colors.eq(colors.clone().map_or(role.colors.clone(), |colors| {
                colors
                    .iter()
                    .map(|&color| Some(color as i32))
                    .collect::<Vec<Option<i32>>>()
            })),
            campsite_role::motion.eq(motion
                .clone()
                .map_or(role.motion, |motion| from_role_motion(motion))),
            campsite_role::displayseparately.eq(raised.unwrap_or(role.display_separately)),
            campsite_role::mentionable.eq(pingable.unwrap_or(role.mentionable)),
            campsite_role::permissions.eq(*permissions as i32),
        ))
        .get_result::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let view = role_view_basic(updated_role);

    event_next_campsite(event_subject, &auth.campsite.id, 0, "RoleUpdated", &view);

    return Ok(Json(view));
}
