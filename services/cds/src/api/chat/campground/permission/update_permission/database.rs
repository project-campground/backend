use cds_schema::schema::cds::role;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::database::DbConnection;

pub fn fetch_role_priorities(
    conn: &mut DbConnection,
    campsite_id: &str,
    executor_roles: &Vec<Uuid>,
    target_roles: &Vec<Uuid>,
) -> Result<Vec<(Uuid, i32)>, diesel::result::Error> {
    let mut roles = role::table
        .filter(
            role::campsite.eq(campsite_id).and(
                role::id
                    .eq_any(executor_roles)
                    .or(role::id.eq_any(target_roles)),
            ),
        )
        .select((role::id, role::priority))
        .load::<(Uuid, i32)>(conn)?;
    roles.sort_by(|a, b| a.cmp(b));

    Ok(roles)
}
