pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campsites::get_campsites,
        get_campsite::get_campsite,
        create_campsite::create_campsite,

        get_bonfire::get_bonfire,
        create_bonfire::create_bonfire,
        update_bonfire::update_bonfire,

        get_members::get_members_given,
        get_members::get_members_any,
    ]
}

mod get_campsites;
mod get_campsite;
mod create_campsite;

mod get_bonfire;
mod create_bonfire;
mod update_bonfire;

mod get_members;