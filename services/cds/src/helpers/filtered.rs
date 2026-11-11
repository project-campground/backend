use multimap::MultiMap;

use crate::helpers::permissions::{PermissionFlag, state::PermissionState};

pub fn get_allowed_and_denied_item_ids<'a, TId, TIter>(
    permissions: &mut TIter,
) -> MultiMap<bool, TId>
where
    TId: 'a + Clone,
    TIter: Iterator<Item = (TId, u64)>,
    TIter: Clone,
{
    let mut map = MultiMap::<bool, TId>::new();

    for permission in permissions {
        let id = permission.0.clone();
        let new_state = PermissionState::from(&permission.1, PermissionFlag::VIEW_CONTENT);

        match new_state {
            PermissionState::Allowed => map.insert(true, id),
            PermissionState::Denied => map.insert(false, id),
            _ => {}
        }
    }

    map
}
#[macro_export]
macro_rules! get_allowed_or_inherited {
    (id: $id: expr, allowed: $allowed: expr, denied: $denied: expr, inherit: $inherit: expr) => {
        // Whether it has easy way out
        $id.eq_any($allowed)
            // Whether it is enabled by default and is not denied
            .or($inherit.into_sql::<Bool>().and(not($id.eq_any($denied))))
    };
}
