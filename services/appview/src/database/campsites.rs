use appview_schema::{
    models::appview::{Campsite, CampsiteMember, CampsiteRole, Profile},
    schema::appview::{campsite, campsite_member, campsite_role, profile},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    database::establish_connection, helpers::api::handle_select_first_error, xrpc::error::XRPCError,
};

pub fn get_campsite_and_member_from_db(
    campsite_id: &str,
    actor: &str,
) -> Result<(Campsite, CampsiteMember), XRPCError> {
    let mut conn = establish_connection().unwrap();
    let camp_member = campsite::table
        .filter(campsite::id.eq(campsite_id))
        .inner_join(
            campsite_member::table.on(campsite_member::campsiteid
                .eq(campsite::id)
                .and(campsite_member::userid.eq(actor))),
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
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq(actor)),
        )
        .left_outer_join(profile::table.on(profile::creator.eq(campsite_member::userid)))
        .first::<(CampsiteMember, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(member)
}
pub fn get_campsite_member(campsite_id: &str, actor: &str) -> Result<CampsiteMember, XRPCError> {
    let mut conn = establish_connection().unwrap();
    let member = campsite_member::table
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(campsite_member::userid.eq(actor)),
        )
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(member)
}

pub fn get_roles_from_db(campsite_id: &str) -> Result<Vec<CampsiteRole>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)
}

pub fn get_specific_roles(role_ids: &Vec<Uuid>) -> Result<Vec<CampsiteRole>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_role::table
        .filter(campsite_role::id.eq_any(role_ids))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)
}
