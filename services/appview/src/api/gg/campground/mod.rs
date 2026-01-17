pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        actor::routes(),
        profile::routes(),
        campsite::routes(),
        tent::routes()
    )
}

mod actor;
mod profile;
mod campsite;
mod tent;