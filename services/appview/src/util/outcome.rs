use rocket::request::Outcome;

pub trait OptionConversionOutcome<T> {
    fn unwrap_outcome<E>(self, none: Outcome<T, E>) -> Outcome<T, E>;
}
pub trait ResultConversionOutcome<T, E> {
    fn outcome<OE>(self, error: fn(E) -> Outcome<T, OE>) -> Outcome<T, OE>;
}

impl<T> OptionConversionOutcome<T> for Option<T> {
    fn unwrap_outcome<E>(self, none: Outcome<T, E>) -> Outcome<T, E> {
        if self.is_none() {
            none
        } else {
            Outcome::Success(self.unwrap())
        }
    }
}
impl<T, E: std::fmt::Debug> ResultConversionOutcome<T, E> for Result<T, E> {
    fn outcome<OE>(self, error: fn(E) -> Outcome<T, OE>) -> Outcome<T, OE> {
        if self.is_ok() {
            Outcome::Success(self.unwrap())
        } else {
            error(self.err().unwrap())
        }
    }
}