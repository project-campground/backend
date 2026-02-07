use std::collections::HashSet;

use appview_schema::{models::appview::{Bonfire, CampsitePermission, CampsiteRole, Tent, TentCategory}, schema::appview::{bonfire, campsite_permission, campsite_role, tent, tent_category}};
use campground_lexicon::gg::campground::tent::{GetTentsOutput, TentCategoryView, TentViewBasic};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection, helpers::{api::{handle_all_db_errors, handle_select_first_error}, permissions::{TentPermissionConsts, aggregate_permissions, fetch_tent_permissions}, tents::{tent_category_view, tent_view_basic}}, util::database::PgBinaryIntegerExpressionMethods, xrpc::{
        campsite::CampsiteInfo, error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getTents?<campsite_id>&<bonfire_id>")]
pub async fn get_tents(auth: CampsiteInfo<'_>, campsite_id: &str, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    if auth.campsite.owner == auth.actor.did {
        return get_tents_unchecked(campsite_id, bonfire_id).await;
    }

    let mut conn = establish_connection().unwrap();  

    // So we can ignore the disallowed permissions if it's home bonfire
    let all_bonfires = bonfire::table
        .filter(
            bonfire::id
                .eq(bonfire_id)
        )
        .load::<Bonfire>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let current_bonfire = all_bonfires.iter().find(|x| x.id == bonfire_id);

    if current_bonfire.is_none() {
        return Err(XRPCError::NotFound);
    }

    let current_bonfire = current_bonfire.unwrap();
    let min_priority = all_bonfires.iter().min_by(|x, y| x.priority.cmp(&y.priority)).unwrap();
    let is_main_group = current_bonfire.id == min_priority.id;

    let member_roles = &auth
        .member
        .roles
        .iter()
        .filter_map(|&x| x)
        .collect::<Vec<Uuid>>();
    
    // Avoid getting too many roles
    let roles = &campsite_role::table
        .filter(
            campsite_role::id
                .eq_any(member_roles)
        )
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;
    let role_tent_permissions = roles
        .iter()
        .fold(0i64, |tent_perm, role|
            tent_perm | role.tent_permissions
        );
    let has_role_permission = role_tent_permissions & TentPermissionConsts::VIEW_CONTENT == TentPermissionConsts::VIEW_CONTENT;
    if !(is_main_group || has_perms_to_view_bonfire(campsite_id, bonfire_id, &auth.actor.did, member_roles, has_role_permission).await?) {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let categories = tent_category::table
        .filter(
            tent_category::campsiteid
                .eq(campsite_id)
                .and(
                    tent_category::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<TentCategory>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();
    let category_ids = &categories.iter().map(|x| x.id).collect::<Vec<Uuid>>();
    let tents = tent::table
        .filter(
            tent::campsiteid
                .eq(campsite_id)
                .and(
                    tent::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<Tent>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();
    let tent_ids = &tents.iter().map(|x| x.id).collect::<Vec<Uuid>>();
    let permissions = campsite_permission::table
        .filter(
            campsite_permission::categoryid
                .eq_any(category_ids)
                .or(
                    campsite_permission::tentid
                        .eq_any(tent_ids)
                )
                .and(
                    campsite_permission::roleid
                        .eq_any(member_roles)
                        .or(
                            campsite_permission::userid
                                .eq(&auth.actor.did)
                        )
                )
                // To not have useless permissions
                .and(
                    campsite_permission::allowedtentpermissions
                        .binary_and(
                            TentPermissionConsts::VIEW_CONTENT
                        )
                        .eq(
                            TentPermissionConsts::VIEW_CONTENT
                        )
                        .or(
                            campsite_permission::deniedtentpermissions
                                .binary_and(
                                    TentPermissionConsts::VIEW_CONTENT
                                )
                                .eq(
                                    TentPermissionConsts::VIEW_CONTENT
                                )
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?;

    // Basically list of IDS that have allowed or denied the permission
    let (category_permissions, tent_permissions): (Vec<&CampsitePermission>, Vec<&CampsitePermission>) = permissions
        .iter()
        .partition(
            |x| x.category_id.is_some()
        );
    let (allowed_categories, denied_categories): (Vec<&CampsitePermission>, Vec<&CampsitePermission>) = category_permissions
        .iter()
        .partition(|x|
            x.allowed_tent_permissions & TentPermissionConsts::VIEW_CONTENT == TentPermissionConsts::VIEW_CONTENT
        );
    let (allowed_tents, denied_tents): (Vec<&CampsitePermission>, Vec<&CampsitePermission>) = tent_permissions
        .iter()
        .partition(|x|
            x.allowed_tent_permissions & TentPermissionConsts::VIEW_CONTENT == TentPermissionConsts::VIEW_CONTENT
        );

    let allowed_categories: HashSet<Uuid> = allowed_categories.iter().map(|x| x.category_id.unwrap())
        .collect();
    let denied_categories: HashSet<Uuid> = denied_categories.iter().map(|x| x.category_id.unwrap())
        .collect();
    let allowed_tents: HashSet<Uuid> = allowed_tents.iter().map(|x| x.tent_id.unwrap())
        .collect();
    let denied_tents: HashSet<Uuid> = denied_tents.iter().map(|x| x.tent_id.unwrap())
        .collect();

    // Since how perms work, tent is viewable if it has permitted role to view it, despite category denying the permission to view
    // Hence, we will ignore its disabled view perm and give the client all categories
    let all_category_ids = categories
        .clone()
        .into_iter()
        .filter(|x|
            allowed_categories.contains(&x.id) ||
            (has_role_permission && !denied_categories.contains(&x.id))
        )
        .map(|x| x.id)
        .collect::<Vec<Uuid>>();
    let all_tents = tents
        .into_iter()
        .filter(|x|
            allowed_tents.contains(&x.id) ||
            (
                (
                    has_role_permission ||
                    x.category_id.map_or(false, |y| all_category_ids.contains(&y))
                )
                && !denied_tents.contains(&x.id)
            )
        )
        .collect::<Vec<TentViewBasic>>();

    return Ok(Json(GetTentsOutput {
        categories,
        tents: all_tents,
    }));
}

async fn has_perms_to_view_bonfire(campsite_id: &str, bonfire_id: &str, actor: &str, role_ids: &Vec<Uuid>, has_role_perm: bool) -> Result<bool> {
    let bonfire_perms = fetch_tent_permissions(campsite_id, bonfire_id, None, None, actor, role_ids).await?;

    let (_, (allowed_tent_permissions, denied_tent_permissions)) = aggregate_permissions(&bonfire_perms);

    // Don't need to check role permissions, because they were overridden
    if allowed_tent_permissions & TentPermissionConsts::VIEW_CONTENT == TentPermissionConsts::VIEW_CONTENT {
        return Ok(true);
    } else if denied_tent_permissions & TentPermissionConsts::VIEW_CONTENT == TentPermissionConsts::VIEW_CONTENT {
        return Ok(false);
    }

    Ok(has_role_perm)
}

async fn get_tents_unchecked(campsite_id: &str, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    let mut conn = establish_connection().unwrap();

    let categories = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::tent_category::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<TentCategory>(&mut conn)
        .expect("Error loading tent categories")
        .iter()
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();
    let tents = crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::tent::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<Tent>(&mut conn)
        .expect("Error loading tents")
        .iter()
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();

    return Ok(Json(GetTentsOutput { tents, categories }));
}