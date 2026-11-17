use rocket::http::uri::{Absolute, Uri};

/// Determines what a passed parameter in HTTP request body field does to object's property usually in the database.
#[derive(Copy, PartialEq, Eq, Debug, Hash)]
pub enum ParamValue<T> {
    /// The new value is assigned to the property of the object
    Passed(T),
    /// Resets the object's property to a default in the database (usually NULL)
    Reset,
    /// The value has not been assigned in HTTP request body and can be ignored
    NotPassed,
}

pub trait AsParamValue<T> {
    fn as_value(self) -> ParamValue<T>;
}
pub trait OptionValidity<T> where Self: Sized {
    fn ensure_validity(self, is_invalid: fn(&T) -> bool) -> Result<Self, ()>;
}

impl<T> OptionValidity<T> for Option<T> {
    fn ensure_validity(self, is_valid: fn(&T) -> bool) -> Result<Option<T>, ()> {
        if self.is_none() {
            return Ok(None);
        }

        let value = self.unwrap();
        if is_valid(&value) {
            Ok(Some(value))
        } else {
            Err(())
        }
    }
}

impl<T> ParamValue<T> {
    /// # Summary
    /// Gets `ParamValue` as `Option` and assigns `ParamValue::NotPassed` values as the given fallback argument.
    /// # Remarks
    /// This is useful for getting `ParamValue` values as optional database column values.
    /// 
    /// - `ParamValue::Passed(T)` will always be returned as `Some(T)`,
    /// - `ParamValue::Reset` will always return `None` and
    /// - `ParamValue::NotPassed` will always return the provided `fallback` argument.
    pub fn with_fallback(self, fallback: Option<T>) -> Option<T> {
        match self {
            ParamValue::Passed(value) => Some(value),
            ParamValue::Reset => None,
            ParamValue::NotPassed => fallback,
        }
    }
    /// Changes the value inside `ParamValue::Passed`, without affecting other `ParamValue` enum values.
    pub fn map_passed<'a, N>(self, map: fn(&T) -> N) -> ParamValue<N> {
        match self {
            ParamValue::Passed(value) => ParamValue::Passed(map(&value)),
            ParamValue::Reset => ParamValue::Reset,
            ParamValue::NotPassed => ParamValue::NotPassed,
        }
    }
    /// Determines whether value in `ParamValue::Passed` is valid based on the passed function and if not, returns `Err(())`.
    /// Every other ParamValue enum values will be returned as `Ok`.
    pub fn ensure_validity(self, check: fn(&T) -> bool) -> Result<ParamValue<T>, ()> {
        match self {
            ParamValue::Passed(value) => if check(&value) { Ok(ParamValue::Passed(value)) } else { Err(()) },
            _ => Ok(self),
        }
    }
}

impl<T> Clone for ParamValue<T>
    where T: Clone
{
    fn clone(&self) -> Self {
        match self {
            ParamValue::Passed(value) => ParamValue::Passed(value.clone()),
            ParamValue::NotPassed => ParamValue::NotPassed,
            ParamValue::Reset => ParamValue::Reset,
        }
    }
}

impl<T, E> ParamValue<Result<T, E>> {
    pub fn reverse(self) -> Result<ParamValue<T>, E> {
        match self {
            ParamValue::Passed(value) =>
                match value {
                    Ok(result) => Ok(ParamValue::Passed(result)),
                    Err(err) => Err(err),  
                },
            ParamValue::Reset => Ok(ParamValue::Reset),
            ParamValue::NotPassed => Ok(ParamValue::NotPassed),
        }
    }
}

impl AsParamValue<String> for Option<String> {
    /// # Summary
    /// Makes an `Option(String)` value for all HTTP request body string-type fields into one of the `ParamValue` enum values.
    /// # Remarks
    /// - As expected, fields that are not provided in the body (are `None`) will be returned as `ParamValue::NotPassed`.
    /// - Empty strings (`""`) will be returned as `ParamValue::Reset` and should ignore any string length or formatting restrictions in the cases where a property can be reset to its default value.
    /// - Non-empty strings (this should be checked with `ensure_validity` method) are returned as `ParamValue::Passed(String)`.
    fn as_value(self) -> ParamValue<String> {
        self.map_or(ParamValue::NotPassed, |x| if x == "" { ParamValue::Reset } else { ParamValue::Passed(x) })
    }
}

impl<'a> AsParamValue<&'a str> for Option<&'a str> {
    /// # Summary
    /// Makes an `Option(&str)` value for all HTTP request body string-type fields into one of the `ParamValue` enum values.
    /// # Remarks
    /// - As expected, fields that are not provided in the body (are `None`) will be returned as `ParamValue::NotPassed`.
    /// - Empty strings (`""`) will be returned as `ParamValue::Reset` and should ignore any string length or formatting restrictions in the cases where a property can be reset to its default value.
    /// - Non-empty strings (this should be checked with `ensure_validity` method) are returned as `ParamValue::Passed(&str)`.
    fn as_value(self) -> ParamValue<&'a str> {
        self.map_or(ParamValue::NotPassed, |x| if x == "" { ParamValue::Reset } else { ParamValue::Passed(x) })
    }
}

pub fn ensure_valid_modified_uri<'a>(value: &'a Option<&str>) -> Result<ParamValue<&'a str>, rocket::http::uri::Error<'a>> {
    if value.is_none() {
        return Ok(ParamValue::NotPassed);
    }

    let value = value.unwrap();

    if value == "" {
        return Ok(ParamValue::Reset);
    }

    let parsed = Uri::<'a>::parse::<Absolute>(value)
        .map(|_| value);

    ParamValue::Passed(parsed).reverse()
}
pub fn ensure_valid_set_uri<'a>(value: &'a Option<&str>) -> Result<Option<&'a str>, rocket::http::uri::Error<'a>> {
    if value.is_none() {
        return Ok(None);
    }

    let value = value.unwrap();

    let parsed = Uri::<'a>::parse::<Absolute>(value)
        .map(|_| Some(value));

    parsed
}
