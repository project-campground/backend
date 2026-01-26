pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_invites::get_invites,
        get_invite::get_invite,
        create_invite::create_invite,
        use_invite::use_invite,
        delete_invite::delete_invite,

        add_member_role::add_member_role,
        remove_member_role::remove_member_role,

        get_members::get_members_given,
        get_members::get_members_any,
        remove_member::remove_member,
        remove_member::remove_self,
        ban_member::ban_member,
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
mod remove_member;
mod ban_member;
mod delete_member_ban;