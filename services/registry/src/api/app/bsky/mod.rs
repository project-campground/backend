pub mod actor;
pub mod feed;
pub mod notification;
pub mod util;

pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.append(&mut actor::routes());
    routes.append(&mut feed::routes());
    routes.append(&mut notification::routes());
    routes
}