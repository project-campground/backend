pub mod resolve_handle;
pub mod update_handle;

pub fn routes() -> Vec<rocket::Route> {
    routes![resolve_handle::resolve_handle, update_handle::update_handle]
}