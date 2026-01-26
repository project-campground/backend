pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        actor::routes(),
        profile::routes(),
        campsite::routes(),
        tent::routes(),
        membership::routes(),
        permission::routes()
    )
}

mod actor;
mod profile;
mod campsite;
mod membership;
mod permission;
mod tent;