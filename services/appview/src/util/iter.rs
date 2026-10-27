use appview_schema::models::appview::{CampsitePermission, CampsiteRole};
use uuid::Uuid;

pub trait HasPermissionOverrides {
    fn get_permissions(&self) -> u64;
}

pub trait AggregatePermissions<T> {
    fn aggregate_permissions(&self) -> T;
}

impl<'a, T> AggregatePermissions<u32> for T
where
    T: Iterator<Item = &'a CampsiteRole> + Clone,
{
    fn aggregate_permissions(&self) -> u32 {
        self.clone().fold(0u32, |a, b| a | b.permissions as u32)
    }
}
impl<'a, TIter, TItem> AggregatePermissions<u64> for TIter
where
    TIter: Iterator<Item = &'a TItem> + Clone,
    TItem: HasPermissionOverrides + 'a,
{
    fn aggregate_permissions(&self) -> u64 {
        // Because denied is 0b10, allowed is 0b11 and denied is 0b00, one can safely just fold by applying |
        self.clone().fold(0u64, |a, b| a | b.get_permissions())
    }
}
impl HasPermissionOverrides for CampsitePermission {
    fn get_permissions(&self) -> u64 {
        self.permissions as u64
    }
}
impl HasPermissionOverrides for (Option<Uuid>, Option<Uuid>, i64) {
    fn get_permissions(&self) -> u64 {
        self.2 as u64
    }
}
