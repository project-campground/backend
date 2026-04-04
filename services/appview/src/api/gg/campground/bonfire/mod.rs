pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_bonfire::get_bonfire,
        create_bonfire::create_bonfire,
        move_bonfire::move_bonfire,
        update_bonfire::update_bonfire,
        delete_bonfire::delete_bonfire,
    ]
}

mod get_bonfire;
mod create_bonfire;
mod move_bonfire;
mod update_bonfire;
mod delete_bonfire;