pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_tent::get_tent,
        get_tents::get_tents,
        create_tent::create_tent,
        update_tent::update_tent,
        delete_tent::delete_tent,
        create_category::create_category,
        update_category::update_category,
        delete_category::delete_category,
        get_messages::get_messages,
        create_message::create_message,
        update_message::update_message,
        delete_message::delete_message,
    ]
}

mod get_tents;
mod get_tent;
mod create_tent;
mod update_tent;
mod delete_tent;
mod create_category;
mod update_category;
mod delete_category;
mod get_messages;
mod create_message;
mod update_message;
mod delete_message;