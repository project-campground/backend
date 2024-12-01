pub mod delete_account;
pub mod get_account_info;
pub mod get_subject_status;
pub mod send_email;
pub mod update_account_email;
pub mod update_account_handle;
pub mod update_account_password;
pub mod update_subject_status;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        delete_account::delete_account,
        get_account_info::get_account_info,
        get_subject_status::get_subject_status,
        send_email::send_email,
        update_account_email::update_account_email,
        update_account_handle::update_account_handle,
        update_account_password::update_account_password,
        update_subject_status::update_subject_status,
    ]
}