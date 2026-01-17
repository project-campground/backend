use diesel::result::Error;

use crate::xrpc::error::XRPCError;

pub fn handle_select_first_error(error: Error) -> XRPCError {
    match error {
        Error::NotFound => XRPCError::NotFound,
        _ => XRPCError::InternalServerError,
    }
}
// pub fn handle_update_error(error: Error) -> XRPCError {
//     match error {
//         _ => XRPCError::InternalServerError,
//     }
// }