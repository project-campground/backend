pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_profile::get_profile,
        get_profiles::get_profiles,
        get_private_profile::get_private_profile,
    ]
}

mod get_private_profile;
mod get_profiles;
mod get_profile;