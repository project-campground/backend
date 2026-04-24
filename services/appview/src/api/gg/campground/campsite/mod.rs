pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campsites::get_campsites,
        get_actor_campsites::get_actor_campsites,
        get_campsite::get_campsite,
        create_campsite::create_campsite,
        update_campsite::update_campsite,
        delete_campsite::delete_campsite,
    ]
}

mod get_actor_campsites;
mod get_campsites;
mod get_campsite;
mod create_campsite;
mod update_campsite;
mod delete_campsite;