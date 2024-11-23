pub mod identity;
pub mod server;
pub mod admin;
pub mod repo;
pub mod sync;

pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.append(&mut identity::routes());
    routes.append(&mut server::routes());
    routes.append(&mut admin::routes());
    routes.append(&mut repo::routes());
    routes.append(&mut sync::routes());
    routes
}