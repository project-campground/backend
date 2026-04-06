pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_roles::get_roles,
        create_role::create_role,
        update_role::update_role,
        move_roles::move_roles,
        delete_role::delete_role,

        add_member_role::add_member_role,
        remove_member_role::remove_member_role,
    ]
}

mod get_roles;
mod create_role;
mod update_role;
mod move_roles;
mod delete_role;

mod add_member_role;
mod remove_member_role;