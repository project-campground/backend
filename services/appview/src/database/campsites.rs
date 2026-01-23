use appview_schema::{models::appview::{Campsite, CampsiteMember, CampsiteRole}, schema::appview::{campsite, campsite_member, campsite_role}};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};

use crate::{database::establish_connection, helpers::api::handle_select_first_error, xrpc::error::XRPCError};

// pub fn get_member_and_roles_from_db(campsite_id: String, actor: String) -> Result<(CampsiteMember, Vec<CampsiteRole>), XRPCError> {
//     let mut conn = establish_connection().unwrap();
//     let member_roles = campsite_member::table
//         .filter(
//             campsite_member::userid
//                 .eq(actor)
//                 .and(
//                     campsite_member::campsiteid
//                         .eq(campsite_id)
//                 )
//         )
//         .left_join(
//             campsite_role::table
//                 .on(
//                     campsite_role::campsiteid
//                         .eq(
//                             campsite_member::campsiteid
//                         )
//                 )
//         )
//         .load::<(CampsiteMember, Option<CampsiteRole>)>(&mut conn)
//         .map_err(handle_select_first_error)?;

//     let member = member_roles.first().ok_or(XRPCError::NotFound)?.0.clone();
//     let roles = member_roles
//         .iter()
//         .filter_map(|x| x.1.clone())
//         .filter(|x| member.roles.contains(&Some(x.id.clone())))
//         .collect::<Vec<CampsiteRole>>();

//     Ok((member, roles))
// }
pub fn get_campsite_and_member_from_db(campsite_id: String, actor: String) -> Result<(Campsite, CampsiteMember), XRPCError> {
    let mut conn = establish_connection().unwrap();
    let camp_member = campsite::table
        .filter(
            campsite::id
                .eq(campsite_id)
        )
        .inner_join(
            campsite_member::table
                .on(
                    campsite_member::campsiteid
                        .eq(
                            campsite::id
                        )
                        .and(
                            campsite_member::userid
                                .eq(actor)
                        )
                )
        )
        .first::<(Campsite, CampsiteMember)>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(camp_member)
}

pub fn get_roles_from_db(campsite_id: String) -> Result<Vec<CampsiteRole>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_role::table
        .filter(
            campsite_role::campsiteid
            .eq(campsite_id.clone())
        )
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)
}