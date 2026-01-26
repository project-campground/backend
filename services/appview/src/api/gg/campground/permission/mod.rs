pub fn routes() -> Vec<rocket::Route> {
    routes![
        update_role_permission::update_tent_role_permission,
        update_role_permission::update_category_role_permission,
        update_role_permission::update_bonfire_role_permission,
        update_user_permission::update_tent_user_permission,
        update_user_permission::update_category_user_permission,
        update_user_permission::update_bonfire_user_permission,
    ]
}


mod update_role_permission;
mod update_user_permission;