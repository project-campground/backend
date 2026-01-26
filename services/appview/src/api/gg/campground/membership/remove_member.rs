use appview_schema::{models::appview::{CampsiteMember, CampsiteRole}, schema::appview::{self, campsite_member, campsite_role}};
use diesel::{BoolExpressionMethods, ExpressionMethods, PgArrayExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::pg::expression::dsl::array_remove;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>&<actor>")]
pub async fn remove_member(auth: CampsiteInfo<'_>, campsite_id: &str, actor: &str) -> Result<()> {    
    if actor == auth.actor.did {
        return remove_self(auth, campsite_id).await;
    }

    if !has_role_perms_or_owner(auth.campsite.clone(), auth.member.clone(), CampsitePermissionConsts::KICK_MEMBERS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let target = campsite_member::table
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(
                    campsite_member::userid.eq(&auth.actor.did)
                )
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    ensure_user_isnt_higher(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), &target.roles, auth.member.roles.clone())?;

    remove_campsite_member(campsite_id, actor)
}
#[post("/xrpc/gg.campground.membership.removeMember?<campsite_id>")]
pub async fn remove_self(auth: CampsiteInfo<'_>, campsite_id: &str) -> Result<()> {    
    remove_campsite_member(campsite_id, &auth.actor.did)
}

pub fn remove_campsite_member(campsite_id: &str, actor: &str) -> Result<()> {
    let mut conn = establish_connection().unwrap();
    diesel::delete(campsite_member::table)
        .filter(
            campsite_member::campsiteid.eq(campsite_id)
                .and(
                    campsite_member::userid
                        .eq(actor)
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::actor::table)
        .filter(
            appview::actor::did.eq(actor)
        )
        .set(
            appview::actor::campsites
                .eq(
                    array_remove(
                        appview::actor::campsites,
                        campsite_id
                    )
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::campsite::table)
        .filter(
            appview::campsite::id.eq(campsite_id)
        )
        .set(
            appview::campsite::memberdids
                .eq(
                    array_remove(
                        appview::campsite::memberdids,
                        actor,
                    )
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(())
}

pub fn ensure_user_isnt_higher(is_owner: bool, all_roles: &mut Vec<CampsiteRole>, target_roles: &Vec<Option<Uuid>>, executor_roles: Vec<Option<Uuid>>) -> Result<(), XRPCError> {
    if is_owner {
        return Ok(());
    }

    all_roles.sort_by(|a, b| a.priority.cmp(&b.priority));

    let executor_highest_role = all_roles
        .iter()
        .find(|x| executor_roles.contains(&Some(x.id)));
    let target_highest_role = all_roles
        .iter()
        .find(|x| target_roles.contains(&Some(x.id)));

    return if executor_highest_role.map_or(
        true,
        |executor_role|
            target_highest_role
                .map_or(false, |target_role| executor_role.priority <= target_role.priority)
    ) {
        Err(XRPCError::Forbidden("Cannot remove target member that has role higher or the same priority as the executing member".to_string()))
    } else {
        Ok(())
    }
}