pub mod bsky;

pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.append(&mut bsky::routes());
    routes
}