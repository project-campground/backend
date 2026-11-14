use campground_lexicon::chat::campground::tent::{
    GetTentsOutput, TentCategoryViewInList, TentType, TentViewInList,
};
use cds_schema::{
    models::cds::{Permission, Tent, TentCategory},
    schema::cds::{permission, role, tent, tent_category},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl, dsl::not,
    sql_types::Bool,
};
use multimap::MultiMap;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::{DbConnection, establish_connection},
    get_allowed_or_inherited,
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        filtered::get_allowed_and_denied_item_ids,
        permissions::{PermissionFlag, state::PermissionState},
    },
    util::iter::AggregatePermissions,
    views::permissions::permission_view_embedded,
    xrpc::{
        campsite::BonfireInfo,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/chat.campground.tent.getTents?<bonfire_id>")]
pub async fn get_tents(auth: BonfireInfo<'_>, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    if auth.campsite.owner == auth.actor.did {
        return unchecked::get_tents(&auth.bonfire.campsite, bonfire_id).await;
    }

    let mut conn = establish_connection().unwrap();

    let member_roles = &auth
        .member
        .roles
        .iter()
        .filter_map(|&x| x)
        .collect::<Vec<Uuid>>();

    let member_has_role_perms = member_has_role_permissions(&mut conn, member_roles)?;

    let permissions = permission::table
        .filter(
            permission::bonfire.eq(bonfire_id).and(
                permission::role
                    .eq_any(member_roles)
                    .or(permission::actor.eq(&auth.actor.did)),
            ),
        )
        .load::<Permission>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let can_view_bonfire = has_perms_to_view_bonfire(member_has_role_perms, &permissions.iter())?;

    if !(auth.bonfire.home || can_view_bonfire) {
        return Err(XRPCError::Forbidden(
            "No given permission to do that".to_string(),
        ));
    }

    // IDs to avoid when fetching in DB
    let categories_state = &get_allowed_and_denied_item_ids(
        &mut permissions
            .iter()
            .filter(|x| x.category.is_some())
            .map(|perm| (perm.category.unwrap(), perm.permissions as u64)),
    );

    // To reduce allocations, but it also increases memory that will not be needed to be allocated
    // Divided by three, so the memory is significantly reduced and only a few reallocations would be needed
    let expected_capacity = permissions.len() / 3;

    let mut tent_permissions = MultiMap::with_capacity(expected_capacity);
    let mut category_permissions = MultiMap::with_capacity(expected_capacity);

    // To not filter like a dozen times
    for permission in permissions.clone() {
        if let Some(category) = permission.category {
            category_permissions.insert(category, permission.clone());
        } else if let Some(tent) = permission.tent {
            tent_permissions.insert(tent, permission);
        }
    }

    let allowed_categories = categories_state
        .get_vec(&true)
        .cloned()
        .unwrap_or_else(|| vec![]);
    let denied_categories = categories_state
        .get_vec(&false)
        .cloned()
        .unwrap_or_else(|| vec![]);

    let categories = tent_category::table
        .filter(
            // Scoping
            tent_category::campsite
                .eq(&auth.bonfire.campsite)
                .and(tent_category::bonfire.eq(bonfire_id))
                .and(get_allowed_or_inherited!(
                    id: tent_category::id,
                    allowed: &allowed_categories,
                    denied: &denied_categories,
                    inherit: can_view_bonfire
                )),
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

    let tent_states = &get_allowed_and_denied_item_ids(
        &mut permissions
            .iter()
            .filter(|item| item.tent.is_some())
            .map(|perm| (perm.tent.unwrap(), perm.permissions as u64)),
    );
    let allowed_tents = tent_states
        .get_vec(&true)
        .cloned()
        .unwrap_or_else(|| vec![]);
    let denied_tents = tent_states
        .get_vec(&false)
        .cloned()
        .unwrap_or_else(|| vec![]);

    let tents = tent::table
        .filter(
            // Scoping
            tent::campsite
                .eq(&auth.bonfire.campsite)
                .and(tent::bonfire.eq(bonfire_id))
                .and(get_allowed_or_inherited!(
                    id: tent::id,
                    allowed: &allowed_tents,
                    denied: &denied_tents,
                    inherit: can_view_bonfire
                )),
        )
        .load::<Tent>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|tent| TentViewInList {
            id: tent.id,
            name: tent.name.clone(),
            category_id: tent.category.clone(),
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
    let role_permissions = &role::table
        .filter(role::id.eq_any(member_roles))
        .select(role::permissions)
        .load::<i32>(conn)
        .map_err(handle_select_first_error)?;

    let member_role_permissions = role_permissions
        .iter()
        .fold(0u32, |initial, &role_permission| {
            initial | (role_permission as u32)
        });

    Ok(member_role_permissions & PermissionFlag::VIEW_CONTENT == PermissionFlag::VIEW_CONTENT)
}

fn has_perms_to_view_bonfire<'a, T>(has_role_perm: bool, all_perms: &T) -> Result<bool>
where
    T: Iterator<Item = &'a Permission>,
    T: Clone,
{
    let bonfire_perms = all_perms
        .clone()
        .filter(|x| x.category.or(x.tent).is_none());
    let perms = bonfire_perms.aggregate_permissions();

    Ok(
        PermissionState::from_single(&perms, PermissionFlag::VIEW_CONTENT)
            .with_condition(has_role_perm),
    )
}
mod unchecked;
