pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_messages::get_messages,
        create_message::create_message,
        update_message::update_message,
        delete_message::delete_message,
    ]
}

mod get_messages;
mod create_message;
mod update_message;
mod delete_message;