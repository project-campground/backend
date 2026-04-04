use std::collections::HashSet;

use appview_schema::{
    models::appview::{CampsitePermission, CampsiteRole, Tent, TentCategory},
    schema::appview::{campsite_permission, campsite_role, tent, tent_category},
};
use campground_lexicon::gg::campground::{
    permission::PermissionViewBasic,
    tent::{GetTentsOutput, TentCategoryView, TentViewBasic},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, dsl::not};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        permissions::ContentPermissionConsts,
    },
    util::iter::AggregatePermissions,
    views::{
        permissions::permission_view_basic,
        tents::{tent_category_view, tent_view_basic},
    },
    xrpc::{
        campsite::BonfireInfo,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.tent.getTents?<bonfire_id>")]
pub async fn get_tents(auth: BonfireInfo<'_>, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    if auth.campsite.owner == auth.actor.did {
        return get_tents_unchecked(&auth.actor.did, &auth.bonfire.campsite_id, bonfire_id).await;
    }

    let mut conn = establish_connection().unwrap();

    let member_roles = &auth
        .member
        .roles
        .iter()
        .filter_map(|&x| x)
        .collect::<Vec<Uuid>>();

    // Avoid getting too many roles
    let roles = &campsite_role::table
        .filter(campsite_role::id.eq_any(member_roles))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;
    let role_content_permissions = roles.iter().fold(0i64, |content_perm, role| {
        content_perm | role.content_permissions
    });
    let has_role_permission = role_content_permissions & ContentPermissionConsts::VIEW_CONTENT
        == ContentPermissionConsts::VIEW_CONTENT;

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
    let member_role_ids = &auth.member.roles;
    let current_member_permissions = &permissions.iter().filter(|x| {
        x.role_id.map_or_else(
            || x.user_id.clone().unwrap() == auth.actor.did,
            |y| member_role_ids.contains(&Some(y)),
        )
    });

    if !(auth.bonfire.home
        || has_perms_to_view_bonfire(
            &auth.actor.did,
            member_roles,
            has_role_permission,
            current_member_permissions,
        )
        .await?)
    {
        return Err(XRPCError::Forbidden(
            "No given permission to do that".to_string(),
        ));
    }

    let (denied_categories, denied_tents) = &get_allowed_denied_tents(current_member_permissions);

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
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();
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
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();

    let permissions = permissions
        .iter()
        .map(permission_view_basic)
        .collect::<Vec<PermissionViewBasic>>();

    return Ok(Json(GetTentsOutput {
        permissions,
        categories,
        tents,
    }));
}

fn get_allowed_denied_tents<'a, T>(permissions: &T) -> (HashSet<Uuid>, HashSet<Uuid>)
where
    T: Iterator<Item = &'a CampsitePermission>,
    T: Clone,
{
    let category_permissions = permissions.clone().filter(|x| x.category_id.is_some());
    let tent_permissions = permissions.clone().filter(|x| x.tent_id.is_some());

    let allowed_categories =
        filter_permissions_and_get_ids(category_permissions.clone().cloned(), false)
            .collect::<HashSet<Uuid>>();
    let denied_categories =
        filter_permissions_and_get_ids(category_permissions.clone().cloned(), true)
            .filter(|x| !allowed_categories.contains(x))
            .collect::<HashSet<Uuid>>();
    let allowed_tents = filter_permissions_and_get_ids(tent_permissions.clone().cloned(), false)
        .collect::<HashSet<Uuid>>();
    let denied_tents = filter_permissions_and_get_ids(tent_permissions.clone().cloned(), true)
        .filter(|x| !allowed_tents.contains(x))
        .collect::<HashSet<Uuid>>();

    (denied_categories, denied_tents)
}
fn filter_permissions_and_get_ids<T>(permissions: T, get_denied: bool) -> impl Iterator<Item = Uuid>
where
    T: Iterator<Item = CampsitePermission>,
    T: Clone,
{
    permissions
        .filter(move |x| {
            (if get_denied {
                x.denied_content_permissions
            } else {
                x.allowed_content_permissions
            }) & ContentPermissionConsts::VIEW_CONTENT
                == ContentPermissionConsts::VIEW_CONTENT
        })
        .map(|x| x.category_id.or(x.tent_id).unwrap())
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

    // Don't need to check role permissions, because they were overridden
    if perms.allowed.content & ContentPermissionConsts::VIEW_CONTENT
        == ContentPermissionConsts::VIEW_CONTENT
    {
        return Ok(true);
    } else if perms.denied.content & ContentPermissionConsts::VIEW_CONTENT
        == ContentPermissionConsts::VIEW_CONTENT
    {
        return Ok(false);
    }

    Ok(has_role_perm)
}

async fn get_tents_unchecked(
    actor: &str,
    campsite_id: &str,
    bonfire_id: &str,
) -> Result<Json<GetTentsOutput>> {
    let mut conn = establish_connection().unwrap();

    let categories = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::tent_category::bonfireid.eq(bonfire_id)),
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
                .and(crate::schema::appview::tent::bonfireid.eq(bonfire_id)),
        )
        .load::<Tent>(&mut conn)
        .expect("Error loading tents")
        .iter()
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();
    let permissions = campsite_permission::table
        .filter(
            campsite_permission::bonfireid.eq(bonfire_id).and(
                campsite_permission::roleid
                    .is_not_null()
                    .or(campsite_permission::userid.eq(&actor)),
            ),
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(permission_view_basic)
        .collect::<Vec<PermissionViewBasic>>();

    return Ok(Json(GetTentsOutput {
        tents,
        categories,
        permissions,
    }));
}
