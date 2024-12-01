pub mod get_actor_likes;
pub mod get_author_feed;
pub mod get_feed;
pub mod get_post_thread;
pub mod get_timeline;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_actor_likes::get_actor_likes,
        get_author_feed::get_author_feed,
        get_feed::get_feed,
        get_post_thread::get_post_thread,
        get_timeline::get_timeline,
    ]
}