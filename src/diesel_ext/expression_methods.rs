use diesel::expression::{AsExpression, Expression};
use diesel::pg::Pg;
use diesel::sql_types::{Double, SqlType};

diesel::infix_operator!(L2Distance, " <-> ", Double, backend: Pg);
diesel::infix_operator!(MaxInnerProduct, " <#> ", Double, backend: Pg);
diesel::infix_operator!(CosineDistance, " <=> ", Double, backend: Pg);
diesel::infix_operator!(L1Distance, " <+> ", Double, backend: Pg);

pub trait VectorExpressionMethods: Expression + Sized {
    fn l2_distance<T>(self, other: T) -> L2Distance<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Self::SqlType>,
    {
        L2Distance::new(self, other.as_expression())
    }

    fn max_inner_product<T>(self, other: T) -> MaxInnerProduct<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Self::SqlType>,
    {
        MaxInnerProduct::new(self, other.as_expression())
    }

    fn cosine_distance<T>(self, other: T) -> CosineDistance<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Self::SqlType>,
    {
        CosineDistance::new(self, other.as_expression())
    }

    fn l1_distance<T>(self, other: T) -> L1Distance<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Self::SqlType>,
    {
        L1Distance::new(self, other.as_expression())
    }
}

impl<T: Expression> VectorExpressionMethods for T {}
