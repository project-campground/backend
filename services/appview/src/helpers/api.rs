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
    println!("Handle all err: {:?}", error);
    XRPCError::InternalServerError
}