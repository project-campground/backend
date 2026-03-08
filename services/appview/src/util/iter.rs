use appview_schema::models::appview::{CampsitePermission, CampsiteRole};
use campground_lexicon::gg::campground::permission::{PermissionsDictionary, PermissionsStateDictionary};

pub trait AggregatePermissions<T> {
    fn aggregate_permissions(&self) -> T;
}

impl<'a, T> AggregatePermissions<PermissionsDictionary> for T
    where T: Iterator<Item = &'a CampsiteRole> + Clone
{
    fn aggregate_permissions(&self) -> PermissionsDictionary {
        let tuple = &mut (0i64, 0i64);
        let (campsite_perm, tent_perm) = self
            .clone()
            .fold(tuple, |perm, role| {
                perm.0 |= role.campsite_permissions;
                perm.1 |= role.tent_permissions;
                perm
            });

        PermissionsDictionary { campsite: campsite_perm.clone(), tent: tent_perm.clone() }
    }
}
impl<'a, T> AggregatePermissions<PermissionsStateDictionary> for T
    where T: Iterator<Item = &'a CampsitePermission> + Clone
{
    fn aggregate_permissions(&self) -> PermissionsStateDictionary {
        // (campsite(allowed, disallowed), tent(allowed, disallowed))
        let aggregated_perms = &mut ((0i64, 0i64), (0i64, 0i64));
        self
        .clone()
        .for_each(|perm| {
            flip_perms(&mut aggregated_perms.0, perm.allowed_campsite_permissions, perm.denied_campsite_permissions);
            flip_perms(&mut aggregated_perms.1, perm.allowed_tent_permissions, perm.denied_tent_permissions);
        });
        
        PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                campsite: aggregated_perms.0.0,
                tent: aggregated_perms.1.0,
            },
            denied: PermissionsDictionary {
                campsite: aggregated_perms.0.1,
                tent: aggregated_perms.1.1,
            }
        }
    }
}

pub fn aggregate_permissions_double_ref<'a, T>(permissions: T) -> PermissionsStateDictionary
    where T: Iterator<Item = &'a &'a CampsitePermission> + Clone
{
        // (campsite(allowed, disallowed), tent(allowed, disallowed))
    let aggregated_perms = &mut ((0i64, 0i64), (0i64, 0i64));
    permissions
        .clone()
        .for_each(|perm| {
            flip_perms(&mut aggregated_perms.0, perm.allowed_campsite_permissions, perm.denied_campsite_permissions);
            flip_perms(&mut aggregated_perms.1, perm.allowed_tent_permissions, perm.denied_tent_permissions);
        });

    PermissionsStateDictionary {
        allowed: PermissionsDictionary {
            campsite: aggregated_perms.0.0,
            tent: aggregated_perms.1.0,
        },
        denied: PermissionsDictionary {
            campsite: aggregated_perms.0.1,
            tent: aggregated_perms.1.1,
        }
    }
}
fn flip_perms(tuple: &mut (i64, i64), allowed: i64, disallowed: i64) {
    tuple.0 |= allowed;
    tuple.1 |= disallowed;
}