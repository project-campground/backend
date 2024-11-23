pub mod get_preferences;
pub mod get_profile;
pub mod get_profiles;
pub mod put_preferences;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_preferences::get_preferences,
        get_profile::get_profile,
        get_profiles::get_profiles,
        put_preferences::put_preferences,
    ]
}