use appview_schema::{models::appview::Bonfire, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::{
    bonfire::BonfireViewInList, permission::PermissionViewEmbedded,
};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl, dsl::not,
    sql_types::Bool,
};
use multimap::MultiMap;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::{DbConnection, campsites::get_role_permissions_from_db, establish_connection},
    get_allowed_or_inherited,
    helpers::{
        api::handle_all_db_errors, filtered::get_allowed_and_denied_item_ids,
        permissions::PermissionFlag,
    },
    xrpc::{campsite::CampsiteInfo, error::Result},
};

#[get("/xrpc/gg.campground.bonfire.getBonfires?<campsite_id>")]
pub async fn get_bonfires(
    auth: CampsiteInfo<'_>,
    campsite_id: &str,
) -> Result<Json<Vec<BonfireViewInList>>> {
    if auth.member.user_id == auth.campsite.owner {
        return get_bonfire_no_checks(campsite_id).await;
    }

    let mut conn = establish_connection().unwrap();

    let roles = auth
        .member
        .roles
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<_>>();

    let (bonfires, bonfire_perms) = conn
        .transaction(|conn| {
            let role_permissions = get_role_permissions_from_db(conn, campsite_id, &roles)?;
            let role_allows =
                (role_permissions & PermissionFlag::VIEW_CONTENT) == PermissionFlag::VIEW_CONTENT;

            let fetched_permissions =
                get_bonfire_perms(conn, campsite_id, &roles, &auth.member.user_id)?;

            let bonfire_states = &get_allowed_and_denied_item_ids(
                &mut fetched_permissions
                    .iter()
                    .map(|perm| (perm.0.clone(), perm.3 as u64)),
            );

            let mut permissions = MultiMap::<String, (Option<Uuid>, Option<String>, u64)>::new();

            for (id, role, user, perms) in fetched_permissions {
                permissions.insert(id, (role, user, perms as u64));
            }

            let allowed_bonfires = bonfire_states
                .get_vec(&true)
                .cloned()
                .unwrap_or_else(|| vec![]);
            let denied_bonfires = bonfire_states
                .get_vec(&false)
                .cloned()
                .unwrap_or_else(|| vec![]);

            Ok((
                crate::schema::appview::bonfire::table
                    .filter(
                        // Scoping
                        crate::schema::appview::bonfire::campsiteid
                            .eq(campsite_id)
                            .and(
                                // Home always displayed
                                crate::schema::appview::bonfire::home.or(
                                    get_allowed_or_inherited!(
                                        id: crate::schema::appview::bonfire::id,
                                        allowed: allowed_bonfires,
                                        denied: denied_bonfires,
                                        inherit: role_allows
                                    ),
                                ),
                            ),
                    )
                    .load::<Bonfire>(conn)?,
                permissions,
            ))
        })
        .map_err(handle_all_db_errors)?;

    let bonfire_views = bonfires
        .iter()
        .map(|bonfire| {
            let permissions = bonfire_perms.get_vec(&bonfire.id).map_or_else(
                || vec![],
                |permissions| {
                    permissions
                        .iter()
                        .map(|perm| PermissionViewEmbedded {
                            role_id: perm.0.clone(),
                            user_id: perm.1.clone(),
                            permissions: (perm.2 as u64).to_string(),
                        })
                        .collect::<Vec<_>>()
                },
            );

            BonfireViewInList {
                id: bonfire.id.clone(),
                name: bonfire.name.clone(),
                description: bonfire.description.clone(),
                avatar_uri: bonfire.avatar_uri.clone(),
                banner_uri: bonfire.banner_uri.clone(),
                position: bonfire.priority,
                home: bonfire.home,
                permissions,
            }
        })
        .collect::<Vec<_>>();

    return Ok(Json(bonfire_views));
}

async fn get_bonfire_no_checks(campsite_id: &str) -> Result<Json<Vec<BonfireViewInList>>> {
    let mut conn = establish_connection().unwrap();

    let bonfires = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(campsite_id))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(|bonfire| BonfireViewInList {
            id: bonfire.id.clone(),
            name: bonfire.name.clone(),
            description: bonfire.description.clone(),
            avatar_uri: bonfire.avatar_uri.clone(),
            banner_uri: bonfire.banner_uri.clone(),
            position: bonfire.priority,
            home: bonfire.home,
            permissions: vec![],
        })
        .collect::<Vec<_>>();

    return Ok(Json(bonfires));
}

fn get_bonfire_perms(
    conn: &mut DbConnection,
    campsite_id: &str,
    actor_roles: &Vec<Uuid>,
    actor: &str,
) -> Result<Vec<(String, Option<Uuid>, Option<String>, i64)>, diesel::result::Error> {
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(campsite_id)
                // Of who
                .and(
                    campsite_permission::roleid
                        .eq_any(actor_roles)
                        .or(campsite_permission::userid.eq(actor)),
                )
                // Where
                .and(
                    campsite_permission::tentid
                        .is_null()
                        .and(campsite_permission::categoryid.is_null()),
                ),
        )
        // Reduce required parsing
        .select((
            campsite_permission::bonfireid,
            campsite_permission::roleid,
            campsite_permission::userid,
            campsite_permission::permissions,
        ))
        .load::<(String, Option<Uuid>, Option<String>, i64)>(conn)
}
