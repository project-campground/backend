pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_post::get_post,
        get_posts::get_posts
    ]
}

mod get_posts;
mod get_post;