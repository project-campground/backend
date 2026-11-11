pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_invites::get_invites,
        get_invite::get_invite,
        create_invite::create_invite,
        use_invite::use_invite,
        delete_invite::delete_invite,
    ]
}


mod get_invites;
mod get_invite;
mod create_invite;
mod use_invite;
mod delete_invite;