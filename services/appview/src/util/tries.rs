#[macro_export]
macro_rules! try_or_continue {
    ($exp: expr) => {
        match $exp {
            Some(x) => x,
            None => {
                return WebSocketOutput::Ignore;
            }
        }
    }
}