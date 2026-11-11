use diesel::{define_sql_function, sql_types::{SingleValue, SqlType}};

define_sql_function! {
    #[aggregate]
    fn array_agg<T: SqlType + SingleValue>(x: T) -> Nullable<Array<T>>;
}