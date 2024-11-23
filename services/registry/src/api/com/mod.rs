pub mod atproto;

pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.append(&mut atproto::routes());
    routes
}