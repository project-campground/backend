use campground_lexicon::chat::campground::permission::PermissionViewDetailed;
use cds_schema::{
    models::cds::{Actor, CampsiteMember, Permission, Profile, Role},
    schema::cds::{actor, campsite_member, permission, profile, role},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    PgExpressionMethods, QueryDsl, RunQueryDsl, sql_types::Bool,
};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
    },
    views::permissions::permission_view_detailed,
    xrpc::{campsite::OneOfInfo, error::Result},
};

#[derive(FromForm)]
pub struct GetPermissionsQuery {
    #[allow(dead_code)]
    tent_id: Option<Uuid>,
    #[allow(dead_code)]
    category_id: Option<Uuid>,
    #[allow(dead_code)]
    bonfire_id: Option<String>,
    non_self: Option<bool>,
}

macro_rules! get_scoped_permissions {
    (campsite: $campsite: expr, bonfire: $bonfire: expr, category: $category: expr, tent: $tent: expr, member: $member: expr, non_self: $non_self: expr) => {{
        // Since it depends from where you want to get permissions
        expect_permission!(
            PermissionCheck::member(&$member)
                .is_owner(&$campsite)
                .skip_if_owner(|permission| scoped_permission(
                    permission.fetch_roles(),
                    &$campsite.id,
                    $bonfire,
                    $category,
                    $tent
                )
                .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_TENTS))?
        );
        let mut conn = establish_connection().unwrap();

        let permissions = permission::table
            .filter(
                permission::campsite
                    .eq(&$campsite.id)
                    .and(permission::bonfire.eq($bonfire))
                    .and(permission::category.is_not_distinct_from($category))
                    .and(permission::tent.is_not_distinct_from($tent))
                    .and(
                        permission::role
                            .is_null()
                            .and(permission::actor.ne(&$member.actor))
                            .or::<bool, Bool>(!$non_self),
                    ),
            )
            // Generally for more details
            .left_join(
                role::table.on(permission::role
                    .is_not_null()
                    .and(role::id.eq(permission::role.assume_not_null()))),
            )
            // All 3 tables for members
            .left_join(
                campsite_member::table.on(permission::actor
                    .is_not_null()
                    .and(campsite_member::actor.eq(permission::actor.assume_not_null()))),
            )
            .left_join(
                profile::table.on(permission::actor.is_not_null().and(
                    profile::actor
                        .eq(permission::actor.assume_not_null())
                        .assume_not_null(),
                )),
            )
            .left_join(
                actor::table.on(permission::actor
                    .is_not_null()
                    .and(actor::did.eq(permission::actor.assume_not_null()))),
            )
            .load::<(
                Permission,
                Option<Role>,
                Option<CampsiteMember>,
                Option<Profile>,
                Option<Actor>,
            )>(&mut conn)
            .map_err(handle_all_db_errors)?
            .iter()
            .map(|(permission, role, member, profile, actor)| {
                permission_view_detailed(
                    permission,
                    role.as_ref(),
                    actor
                        .as_ref()
                        .map(|actor| (actor, profile.as_ref(), member.as_ref())),
                )
            })
            .collect::<Vec<PermissionViewDetailed>>();

        Ok(Json(permissions))
    }};
}

#[get("/xrpc/chat.campground.permission.getPermissions?<query..>", rank = 1)]
pub async fn get_permissions(
    auth: OneOfInfo<'_>,
    query: GetPermissionsQuery,
) -> Result<Json<Vec<PermissionViewDetailed>>> {
    let non_self = query.non_self.unwrap_or(false);

    match auth {
        OneOfInfo::Tent(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.tent.bonfire,
                category: auth.tent.category.as_ref(),
                tent: Some(&auth.tent.id),
                member: auth.member,
                non_self: non_self
            )
        }
        OneOfInfo::Category(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.category.bonfire,
                category: Some(&auth.category.id),
                tent: Option::<&Uuid>::None,
                member: auth.member,
                non_self: non_self
            )
        }
        OneOfInfo::Bonfire(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.bonfire.id,
                category: Option::<&Uuid>::None,
                tent: Option::<&Uuid>::None,
                member: auth.member,
                non_self: non_self
            )
        }
    }
}
