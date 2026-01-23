pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campsites::get_campsites,
        get_campsite::get_campsite,
        create_campsite::create_campsite,

        get_bonfire::get_bonfire,
        create_bonfire::create_bonfire,
        update_bonfire::update_bonfire,
        delete_bonfire::delete_bonfire,

        get_roles::get_roles,
        create_role::create_role,
        add_member_role::add_member_role,
        remove_member_role::remove_member_role,
        update_role::update_role,
        delete_role::delete_role,

        get_members::get_members_given,
        get_members::get_members_any,
    ]
}

mod get_campsites;
mod get_campsite;
mod create_campsite;

mod get_roles;
mod create_role;
mod add_member_role;
mod remove_member_role;
mod update_role;
mod delete_role;

mod get_bonfire;
mod create_bonfire;
mod update_bonfire;
mod delete_bonfire;

mod get_members;