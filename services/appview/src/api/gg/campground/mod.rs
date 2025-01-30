pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        actor::routes()
    )
}

mod actor;