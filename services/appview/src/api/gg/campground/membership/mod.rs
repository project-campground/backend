pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_invites::get_invites,
        get_invite::get_invite,
        create_invite::create_invite,
        use_invite::use_invite,
        delete_invite::delete_invite,

        add_member_role::add_member_role,
        remove_member_role::remove_member_role,

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


mod get_invites;
mod get_invite;
mod create_invite;
mod use_invite;
mod delete_invite;

mod add_member_role;
mod remove_member_role;

mod get_members;
mod get_members_detailed;
mod get_member;
mod update_member;
mod remove_member;

mod get_member_ban;
mod get_member_bans;
mod ban_member;
mod delete_member_ban;