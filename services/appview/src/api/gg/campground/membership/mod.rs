pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_member::get_member,
        get_members::get_members_given,
        get_members::get_members_any,
        get_members_detailed::get_members_detailed,
        update_member::update_member,
        remove_member::remove_member,

        get_member_bans::get_member_bans,
        get_member_ban::get_member_ban,
        ban_member::ban_member,
        delete_member_ban::delete_member_ban,
    ]
}

mod get_members;
mod get_members_detailed;
mod get_member;
mod update_member;
mod remove_member;

mod get_member_ban;
mod get_member_bans;
mod ban_member;
mod delete_member_ban;