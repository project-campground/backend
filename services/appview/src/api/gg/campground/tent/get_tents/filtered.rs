use std::collections::HashSet;

use appview_schema::models::appview::CampsitePermission;
use uuid::Uuid;

use crate::helpers::permissions::{PermissionConsts, state::PermissionState};

pub fn get_denied_item_ids<'a, T, Filter: FnMut(&&'a CampsitePermission) -> bool>(
    permissions: &T,
    filter_by: Filter,
) -> HashSet<Uuid>
where
    T: Iterator<Item = &'a CampsitePermission>,
    T: Clone,
    Filter: Clone,
{
    let specific_permissions = permissions.clone().filter(filter_by);

    let allowed_items = filter_permissions_and_get_ids(
        specific_permissions.clone().cloned(),
        PermissionState::Allowed,
    )
    .collect::<HashSet<Uuid>>();

    let denied_items = filter_permissions_and_get_ids(
        specific_permissions.clone().cloned(),
        PermissionState::Denied,
    )
    .filter(|x| !allowed_items.contains(x))
    .collect::<HashSet<Uuid>>();

    denied_items
}

fn filter_permissions_and_get_ids<T>(
    permissions: T,
    expected_value: PermissionState,
) -> impl Iterator<Item = Uuid>
where
    T: Iterator<Item = CampsitePermission>,
    T: Clone,
{
    permissions
        .filter(move |x| {
            PermissionState::from_single(&(x.permissions as u64), PermissionConsts::VIEW_CONTENT)
                == expected_value
        })
        .map(|x| x.category_id.or(x.tent_id).unwrap())
}
