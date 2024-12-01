pub mod register_push;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        register_push::register_push
    ]
}