pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        actor::routes(),
        bonfire::routes(),
        campsite::routes(),
        invite::routes(),
        membership::routes(),
        message::routes(),
        permission::routes(),
        profile::routes(),
        role::routes(),
        tent::routes()
    )
}

mod actor;
mod bonfire;
mod campsite;
mod invite;
mod membership;
mod message;
mod permission;
mod profile;
mod role;
mod tent;