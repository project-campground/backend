pub fn routes() -> Vec<rocket::Route> {
    merge_routes!(
        campground::routes()
    )
}

mod campground;