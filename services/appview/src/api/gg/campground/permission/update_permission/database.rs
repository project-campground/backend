use appview_schema::schema::appview::campsite_role;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::database::DbConnection;

pub fn fetch_role_priorities(
    conn: &mut DbConnection,
    campsite_id: &str,
    executor_roles: &Vec<Uuid>,
    target_roles: &Vec<Uuid>,
) -> Result<Vec<(Uuid, i32)>, diesel::result::Error> {
    let mut roles = campsite_role::table
        .filter(
            campsite_role::campsiteid.eq(campsite_id).and(
                campsite_role::id
                    .eq_any(executor_roles)
                    .or(campsite_role::id.eq_any(target_roles)),
            ),
        )
        .select((campsite_role::id, campsite_role::priority))
        .load::<(Uuid, i32)>(conn)?;
    roles.sort_by(|a, b| a.cmp(b));

    Ok(roles)
}
