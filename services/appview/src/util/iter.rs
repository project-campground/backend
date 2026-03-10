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
        let (general_perm, content_perm) = self
            .clone()
            .fold(tuple, |perm, role| {
                perm.0 |= role.general_permissions;
                perm.1 |= role.content_permissions;
                perm
            });

        PermissionsDictionary { general: general_perm.clone(), content: content_perm.clone() }
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
            flip_perms(&mut aggregated_perms.0, perm.allowed_general_permissions, perm.denied_general_permissions);
            flip_perms(&mut aggregated_perms.1, perm.allowed_content_permissions, perm.denied_content_permissions);
        });
        
        PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                general: aggregated_perms.0.0,
                content: aggregated_perms.1.0,
            },
            denied: PermissionsDictionary {
                general: aggregated_perms.0.1,
                content: aggregated_perms.1.1,
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
            flip_perms(&mut aggregated_perms.0, perm.allowed_general_permissions, perm.denied_general_permissions);
            flip_perms(&mut aggregated_perms.1, perm.allowed_content_permissions, perm.denied_content_permissions);
        });

    PermissionsStateDictionary {
        allowed: PermissionsDictionary {
            general: aggregated_perms.0.0,
            content: aggregated_perms.1.0,
        },
        denied: PermissionsDictionary {
            general: aggregated_perms.0.1,
            content: aggregated_perms.1.1,
        }
    }
}
fn flip_perms(tuple: &mut (i64, i64), allowed: i64, disallowed: i64) {
    tuple.0 |= allowed;
    tuple.1 |= disallowed;
}