pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_post::get_post,
        get_post::get_post_default_limit,
        get_post::get_post_default_offset,
        get_post::get_post_default,
        unindex_post::unindex_post,
        get_posts::get_posts,
        get_posts::get_posts_defaults,
        get_posts::get_posts_defaults_with_offset,
        get_replies::get_replies,
        get_replies::get_replies_default_limit,
        get_replies::get_replies_default_offset,
        get_replies::get_replies_default
    ]
}

mod get_posts;
mod get_replies;
mod get_post;
mod unindex_post;