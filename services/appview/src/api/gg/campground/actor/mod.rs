pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_me::get_me,
        get_profile::get_profile,
        get_profiles::get_profiles
    ]
}

mod get_profiles;
mod get_profile;
mod get_me;