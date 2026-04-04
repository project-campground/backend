pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_tent::get_tent,
        get_tents::get_tents,
        create_tent::create_tent,
        update_tent::update_tent,
        move_tent::move_tent,
        delete_tent::delete_tent,
        create_category::create_category,
        update_category::update_category,
        move_category::move_category,
        delete_category::delete_category,
    ]
}

mod get_tents;
mod get_tent;
mod create_tent;
mod update_tent;
mod move_tent;
mod delete_tent;

mod create_category;
mod update_category;
mod move_category;
mod delete_category;