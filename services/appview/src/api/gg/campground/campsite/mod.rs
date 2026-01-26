pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campsites::get_campsites,
        get_campsite::get_campsite,
        create_campsite::create_campsite,
        update_campsite::update_campsite,

        get_bonfire::get_bonfire,
        create_bonfire::create_bonfire,
        update_bonfire::update_bonfire,
        delete_bonfire::delete_bonfire,

        get_roles::get_roles,
        create_role::create_role,
        update_role::update_role,
        delete_role::delete_role,
    ]
}

mod get_campsites;
mod get_campsite;
mod create_campsite;
mod update_campsite;

mod get_roles;
mod create_role;
mod update_role;
mod delete_role;

mod get_bonfire;
mod create_bonfire;
mod update_bonfire;
mod delete_bonfire;