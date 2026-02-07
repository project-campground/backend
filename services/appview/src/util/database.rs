use diesel::{Expression, dsl::AsExprOf, expression::AsExpression, infix_operator, pg::Pg, sql_types::{BigInt, Integer, SqlType}};

infix_operator!(BinaryAnd, " & ", BigInt, backend: Pg);

pub trait PgBinaryIntegerExpressionMethods: Expression + Sized {
    fn binary_and<T>(self, other: T) -> BinaryAnd<Self, AsExprOf<T, BigInt>>
    where
        Self::SqlType: SqlType,
        T: AsExpression<BigInt>,
    {
        BinaryAnd::new(self, other.as_expression())
    }
}

pub trait IntegerOrBigInteger {}

impl IntegerOrBigInteger for Integer {}
impl IntegerOrBigInteger for BigInt {}

impl<T> PgBinaryIntegerExpressionMethods for T
where
    T: Expression,
    T::SqlType: IntegerOrBigInteger,
{
}