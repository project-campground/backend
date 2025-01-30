pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_profile::get_profile
    ]
}

mod get_profile;