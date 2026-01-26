use diesel::result::Error;

use crate::xrpc::error::XRPCError;

pub fn handle_select_first_error(error: Error) -> XRPCError {
    match error {
        Error::NotFound => XRPCError::NotFound,
        _ => {
            println!("Err: {:?}", error);
            return XRPCError::InternalServerError;
        },
    }
}
pub fn handle_all_db_errors(error: Error) -> XRPCError {
    println!("Err: {:?}", error);
    XRPCError::InternalServerError
}
// pub fn handle_select_member_error(error: Error) -> XRPCError {
//     match error {
//         Error::NotFound => XRPCError::Forbidden("User not part of the server".to_string()),
//         _ => XRPCError::InternalServerError,
//     }
// }
// pub fn handle_update_error(error: Error) -> XRPCError {
//     match error {
//         _ => XRPCError::InternalServerError,
//     }
// }