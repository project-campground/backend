pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        actor::routes(),
        profile::routes()
    )
}

mod actor;
mod profile;