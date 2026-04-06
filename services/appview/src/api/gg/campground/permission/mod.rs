pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_permissions::get_permissions,
        update_permission::update_permission,
    ]
}


mod get_permissions;
mod update_permission;
mod update_role_permission;
mod update_user_permission;