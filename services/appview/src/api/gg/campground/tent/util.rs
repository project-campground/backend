use appview_schema::models::appview::{Bonfire, Campsite, CampsiteMember, TentCategory};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    database::DbConnection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
    },
    xrpc::error::XRPCError,
};

pub fn check_bonfire_existence(
    conn: &mut DbConnection,
    campsite: &Campsite,
    member: &CampsiteMember,
    moved_bonfire_id: &str,
) -> Result<(), XRPCError> {
    let bonfires = &crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::id
                .eq(moved_bonfire_id)
                .and(crate::schema::appview::bonfire::campsiteid.eq(&campsite.id)),
        )
        .load::<Bonfire>(conn)
        .map_err(handle_select_first_error)?;

    if bonfires.len() == 0 {
        return Err(XRPCError::BadRequest(
            "Bonfire supplied in 'bonfire_id' does not exist".to_string(),
        ));
    }

    expect_permission!(
        PermissionCheck::member(&member)
            .is_owner(&campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &campsite.id,
                &moved_bonfire_id,
                None,
                None
            )
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_TENTS))?
    );

    return Ok(());
}

pub fn check_category_existence(
    conn: &mut DbConnection,
    campsite: &Campsite,
    member: &CampsiteMember,
    category_id: Uuid,
) -> Result<String, XRPCError> {
    let tent_categories = &crate::schema::appview::tent_category::table
        .filter(crate::schema::appview::tent_category::id.eq(category_id))
        .load::<TentCategory>(conn)
        .map_err(handle_select_first_error)?;

    if tent_categories.len() == 0 {
        return Err(XRPCError::NotFound);
    }

    let tent_category = tent_categories.first().unwrap();

    // Can't move tent out of campsite
    if tent_category.campsite_id != campsite.id {
        return Err(XRPCError::NotFound);
    }

    // FIXME: Reduce duplicate fetching for bonfire permissions
    expect_permission!(
        PermissionCheck::member(&member)
            .is_owner(&campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &campsite.id,
                &tent_category.bonfire_id,
                Some(&category_id),
                None
            )
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_TENTS))?
    );

    return Ok(tent_category.bonfire_id.clone());
}

#[macro_export]
macro_rules! make_room_for_items {
    ($table: path, $id_column: path, $priority_column: path, $filter: expr; $conn: expr, $position: expr) => {
        let items_exist_in_pos = $table
            .filter(
                $filter
                    .and($priority_column.eq($position)),
            )
            .count()
            .first::<i64>($conn)?;

        // No other items to update
        if items_exist_in_pos < 1 {
            return Ok(vec![]);
        }

        // Make other tents go below it (since client is expected to add 1 when putting below a tent already)
        return diesel::update($table)
            .filter(
                $filter
                    // Update only item at that position and below
                    .and($priority_column.ge($position))
                    // Make sure it can even go down
                    .and(
                        $priority_column
                            .cast::<BigInt>()
                            .add(1)
                            .le(i32::MAX as i64),
                    ),
            )
            .set(($priority_column.eq($priority_column + 1)))
            .returning(($id_column, $priority_column))
            .load::<(Uuid, i32)>($conn);
    };
}
