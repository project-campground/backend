use appview_schema::{
    models::appview::{Actor, CampsiteMember, CampsitePermission, CampsiteRole, Profile},
    schema::appview::{actor, campsite_member, campsite_permission, campsite_role, profile},
};
use campground_lexicon::gg::campground::permission::{
    GetPermissionsOutput, PermissionViewDetailed,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, sql_types::Bool,
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
    (campsite: $campsite: expr, bonfire: $bonfire: expr, category: $category: expr, tent: $tent: expr, member: $member: expr, non_self: $non_self: expr, permission_filter: $perm_filter: expr) => {{
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

        let permissions = campsite_permission::table
            .filter(
                $perm_filter.and(
                    campsite_permission::roleid
                        .is_null()
                        .and(campsite_permission::userid.ne(&$member.user_id))
                        .or::<bool, Bool>(!$non_self),
                ),
            )
            .left_join(
                campsite_role::table.on(campsite_permission::roleid
                    .is_not_null()
                    .and(campsite_role::id.eq(campsite_permission::roleid.assume_not_null()))),
            )
            .left_join(
                campsite_member::table.on(campsite_permission::userid.is_not_null().and(
                    campsite_member::userid.eq(campsite_permission::userid.assume_not_null()),
                )),
            )
            .left_join(
                profile::table.on(campsite_permission::userid.is_not_null().and(
                    profile::creator
                        .eq(campsite_permission::userid.assume_not_null())
                        .assume_not_null(),
                )),
            )
            .left_join(
                actor::table.on(campsite_permission::userid
                    .is_not_null()
                    .and(actor::did.eq(campsite_permission::userid.assume_not_null()))),
            )
            .load::<(
                CampsitePermission,
                Option<CampsiteRole>,
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

        Ok(Json(GetPermissionsOutput { permissions }))
    }};
}

#[get("/xrpc/gg.campground.permission.getPermissions?<query..>", rank = 1)]
pub async fn get_permissions(
    auth: OneOfInfo<'_>,
    query: GetPermissionsQuery,
) -> Result<Json<GetPermissionsOutput>> {
    let non_self = query.non_self.unwrap_or(false);

    match auth {
        OneOfInfo::Tent(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.tent.bonfire_id,
                category: auth.tent.category_id.as_ref(),
                tent: Some(&auth.tent.id),
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::tentid.eq(auth.tent.id)
            )
        }
        OneOfInfo::Category(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.category.bonfire_id,
                category: Some(&auth.category.id),
                tent: None,
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::categoryid.eq(auth.category.id)
            )
        }
        OneOfInfo::Bonfire(auth) => {
            get_scoped_permissions!(
                campsite: auth.campsite,
                bonfire: &auth.bonfire.id,
                category: None,
                tent: None,
                member: auth.member,
                non_self: non_self,
                permission_filter: campsite_permission::bonfireid.eq(auth.bonfire.id)
            )
        }
    }
}
