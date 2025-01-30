pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_profile::get_profile,
        get_profiles::get_profiles
    ]
}

mod get_profiles;
mod get_profile;