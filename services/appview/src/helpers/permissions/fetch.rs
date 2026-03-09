use appview_schema::{models::appview::CampsitePermission, schema::appview::campsite_permission};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, sql_types::Bool};
use uuid::Uuid;

use crate::{database::establish_connection, helpers::api::{handle_all_db_errors, handle_select_first_error}, xrpc::error::XRPCError};

pub fn fetch_all_campsite_permissions(campsite_id: &str, actor: &str) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                .and(
                    campsite_permission::userid
                        .is_null()
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)
}
/// ## Summary
/// Fetches permissions from bonfire level to the specified category or tent level.
/// 
/// ## Remarks
/// If category and tent IDs are none, it only fetches bonfire-level.
/// If tent is some, then it fetches bonfire and tent level permission.
/// If category is some, then it fetches bonfire and category level permissions.
/// If category and tent are specified, then it fetches from bonfire all the way to tent level, including category-level.
pub async fn fetch_tent_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                // Of who
                .and(
                    campsite_permission::roleid
                        .eq_any(role_ids)
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
                // Where
                .and(
                    campsite_permission::bonfireid
                        .eq(
                            bonfire_id
                        )
                        .and(
                            campsite_permission::tentid
                                .is_null()
                        )
                        .and(
                            campsite_permission::categoryid
                                .is_null()
                        )
                        .or(
                            campsite_permission::tentid
                                .eq(tent_id)
                                .and::<bool, Bool>(tent_id.is_some())
                                .or(
                                    campsite_permission::categoryid
                                        .eq(category_id)
                                        .and::<bool, Bool>(category_id.is_some())
                                )
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)
}
/// ## Summary
/// Fetches only the specified level and none of the above
/// ### Remarks
/// If tent is specified, then only tent-level permissions are fetched.
/// If category is specified, then only category-level permissions are fetched.
/// If none are specified, then only bonfire-level permissions are fetched.
pub async fn fetch_only_specific_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                // Of who
                .and(
                    campsite_permission::roleid
                        .eq_any(role_ids)
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
                // Where
                .and(
                    // Either bonfire
                    campsite_permission::bonfireid
                        .eq(
                            bonfire_id
                        )
                        .and(
                            campsite_permission::tentid
                                .is_null()
                                .and::<bool, Bool>(
                                    tent_id.is_none()
                                )
                        )
                        .and(
                            campsite_permission::categoryid
                                .is_null()
                                .and::<bool, Bool>(
                                    category_id.is_none()
                                )
                        )
                        // Or tent if specified
                        .or(
                            campsite_permission::tentid
                                .is_not_null()
                                .and(
                                    campsite_permission::tentid
                                        .eq(tent_id)
                                )
                        )
                        // Or category if specified
                        .or(
                            campsite_permission::categoryid
                                .is_not_null()
                                .and(
                                    campsite_permission::categoryid
                                        .eq(category_id)
                                )
                            
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)
}