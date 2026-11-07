use appview_schema::{
    models::appview::{CampsitePermission, Tent, TentCategory},
    schema::appview::{campsite_permission, campsite_role, tent, tent_category},
};
use campground_lexicon::gg::campground::tent::{
    GetTentsOutput, TentCategoryViewInList, TentType, TentViewInList,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, dsl::not};
use multimap::MultiMap;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    api::gg::campground::tent::get_tents::filtered::get_denied_item_ids,
    database::{DbConnection, establish_connection},
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        permissions::{PermissionFlag, state::PermissionState},
    },
    util::iter::AggregatePermissions,
    views::permissions::permission_view_embedded,
    xrpc::{
        campsite::BonfireInfo,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.tent.getTents?<bonfire_id>")]
pub async fn get_tents(auth: BonfireInfo<'_>, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    if auth.campsite.owner == auth.actor.did {
        return unchecked::get_tents(&auth.bonfire.campsite_id, bonfire_id).await;
    }

    let mut conn = establish_connection().unwrap();

    let member_roles = &auth
        .member
        .roles
        .iter()
        .filter_map(|&x| x)
        .collect::<Vec<Uuid>>();

    let member_has_role_perms = member_has_role_permissions(&mut conn, member_roles)?;

    let permissions = campsite_permission::table
        .filter(
            campsite_permission::bonfireid.eq(bonfire_id).and(
                campsite_permission::roleid
                    .is_not_null()
                    .or(campsite_permission::userid.eq(&auth.actor.did)),
            ),
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let relevant_member_permissions = permissions.iter().filter(|x| {
        x.role_id.map_or_else(
            || x.user_id.clone().unwrap() == auth.actor.did,
            |y| member_roles.contains(&y),
        )
    });

    if !(auth.bonfire.home
        || has_perms_to_view_bonfire(
            &auth.actor.did,
            member_roles,
            member_has_role_perms,
            &relevant_member_permissions,
        )
        .await?)
    {
        return Err(XRPCError::Forbidden(
            "No given permission to do that".to_string(),
        ));
    }

    // IDs to avoid when fetching in DB
    let denied_categories = &get_denied_item_ids(&relevant_member_permissions, |item| {
        item.category_id.is_some()
    });

    // To reduce allocations, but it also increases memory that will not be needed to be allocated
    // Divided by three, so the memory is significantly reduced and only a few reallocations would be needed
    let expected_capacity = permissions.len() / 3;

    let mut tent_permissions = MultiMap::with_capacity(expected_capacity);
    let mut category_permissions = MultiMap::with_capacity(expected_capacity);

    // To not filter like a dozen times
    for permission in permissions.clone() {
        if let Some(category_id) = permission.category_id {
            category_permissions.insert(category_id, permission.clone());
        }

        let tent = permission.tent_id.clone().unwrap();

        tent_permissions.insert(tent, permission);
    }

    let categories = tent_category::table
        .filter(
            tent_category::campsiteid
                .eq(&auth.bonfire.campsite_id)
                .and(tent_category::bonfireid.eq(bonfire_id))
                .and(not(tent_category::id.eq_any(denied_categories))),
        )
        .load::<TentCategory>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|category| TentCategoryViewInList {
            id: category.id,
            name: category.name.to_owned(),
            description: category.description.to_owned(),
            position: category.priority,
            permissions: category_permissions.get_vec(&category.id).map_or_else(
                || vec![],
                |category_permissions| {
                    category_permissions
                        .iter()
                        .map(permission_view_embedded)
                        .collect::<Vec<_>>()
                },
            ),
        })
        .collect::<Vec<_>>();

    let denied_tents =
        &get_denied_item_ids(&relevant_member_permissions, |item| item.tent_id.is_some());
    let tents = tent::table
        .filter(
            tent::campsiteid
                .eq(&auth.bonfire.campsite_id)
                .and(tent::bonfireid.eq(bonfire_id))
                .and(
                    tent::categoryid
                        .is_null()
                        .or(not(tent::categoryid.eq_any(denied_categories))),
                )
                .and(not(tent::id.eq_any(denied_tents))),
        )
        .load::<Tent>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|tent| TentViewInList {
            id: tent.id,
            name: tent.name.clone(),
            category_id: tent.category_id.clone(),
            r#type: TentType::Text,
            view_type: 0,
            description: tent.description.to_owned(),
            position: tent.priority,
            permissions: tent_permissions.get_vec(&tent.id).map_or_else(
                || vec![],
                |tent_permissions| {
                    tent_permissions
                        .iter()
                        .map(permission_view_embedded)
                        .collect::<Vec<_>>()
                },
            ),
        })
        .collect::<Vec<_>>();

    return Ok(Json(GetTentsOutput { categories, tents }));
}

fn member_has_role_permissions(
    conn: &mut DbConnection,
    member_roles: &Vec<Uuid>,
) -> Result<bool, XRPCError> {
    let role_permissions = &campsite_role::table
        .filter(campsite_role::id.eq_any(member_roles))
        .select(campsite_role::permissions)
        .load::<i32>(conn)
        .map_err(handle_select_first_error)?;

    let member_role_permissions = role_permissions
        .iter()
        .fold(0u32, |initial, &role_permission| {
            initial | (role_permission as u32)
        });

    Ok(member_role_permissions & PermissionFlag::VIEW_CONTENT == PermissionFlag::VIEW_CONTENT)
}

async fn has_perms_to_view_bonfire<'a, T>(
    actor: &str,
    role_ids: &Vec<Uuid>,
    has_role_perm: bool,
    all_perms: &T,
) -> Result<bool>
where
    T: Iterator<Item = &'a CampsitePermission>,
    T: Clone,
{
    let bonfire_perms = all_perms.clone().filter(|x| {
        x.category_id.or(x.tent_id).is_none()
            && x.role_id.map_or_else(
                || x.user_id.clone().unwrap() == actor,
                |y| role_ids.contains(&y),
            )
    });
    let perms = bonfire_perms.aggregate_permissions();

    Ok(
        PermissionState::from_single(&perms, PermissionFlag::VIEW_CONTENT).map_non_denied(|| {
            if has_role_perm {
                PermissionState::Allowed
            } else {
                PermissionState::Denied
            }
        }) == PermissionState::Allowed,
    )
}
mod filtered;
mod unchecked;
