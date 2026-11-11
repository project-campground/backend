pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_post::get_post,
        unindex_post::unindex_post,
        get_posts::get_posts,
        get_replies::get_replies,
    ]
}

mod get_posts;
mod get_replies;
mod get_post;
mod unindex_post;