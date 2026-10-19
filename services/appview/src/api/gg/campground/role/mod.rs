pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_roles::get_roles,
        create_role::create_role,
        update_role::update_role,
        move_role::move_role,
        delete_role::delete_role,
        add_member_role::add_member_role,
        remove_member_role::remove_member_role,
    ]
}

mod create_role;
mod delete_role;
mod get_roles;
mod move_role;
mod update_role;

mod add_member_role;
mod remove_member_role;
