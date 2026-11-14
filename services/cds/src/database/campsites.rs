use cds_schema::{
    models::cds::{Campsite, CampsiteMember, Profile, Role},
    schema::cds::{campsite, campsite_member, profile, role},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    database::{DbConnection, establish_connection},
    helpers::api::handle_select_first_error,
    xrpc::error::XRPCError,
};

pub fn get_campsite_and_member_from_db(
    campsite_id: &str,
    actor: &str,
) -> Result<(Campsite, CampsiteMember), XRPCError> {
    let mut conn = establish_connection().unwrap();
    let camp_member = campsite::table
        .filter(campsite::id.eq(campsite_id))
        .inner_join(
            campsite_member::table.on(campsite_member::campsite
                .eq(campsite::id)
                .and(campsite_member::actor.eq(actor))),
        )
        .first::<(Campsite, CampsiteMember)>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(camp_member)
}
pub fn get_full_campsite_member(
    campsite_id: &str,
    actor: &str,
) -> Result<(CampsiteMember, Option<Profile>), XRPCError> {
    let mut conn = establish_connection().unwrap();
    let member = campsite_member::table
        .filter(
            campsite_member::campsite
                .eq(campsite_id)
                .and(campsite_member::actor.eq(actor)),
        )
        .left_outer_join(profile::table.on(profile::actor.eq(campsite_member::actor)))
        .first::<(CampsiteMember, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(member)
}
pub fn get_campsite_member(campsite_id: &str, actor: &str) -> Result<CampsiteMember, XRPCError> {
    let mut conn = establish_connection().unwrap();
    let member = campsite_member::table
        .filter(
            campsite_member::campsite
                .eq(campsite_id)
                .and(campsite_member::actor.eq(actor)),
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(member)
}

pub fn get_roles_from_db(campsite_id: &str) -> Result<Vec<Role>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    role::table
        .filter(role::campsite.eq(campsite_id))
        .load::<Role>(&mut conn)
        .map_err(handle_select_first_error)
}
pub fn get_role_permissions_from_db(
    conn: &mut DbConnection,
    campsite_id: &str,
    role_ids: &Vec<Uuid>,
) -> Result<u32, diesel::result::Error> {
    Ok(role::table
        .filter(
            role::campsite
                .eq(campsite_id)
                .and(role::id.eq_any(role_ids)),
        )
        .select(role::permissions)
        .load::<i32>(conn)?
        .iter()
        .fold(0u32, |initial, permission| initial | (*permission as u32)))
}
