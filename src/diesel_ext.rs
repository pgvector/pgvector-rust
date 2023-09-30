use diesel::deserialize::{self, FromSql};
use diesel::expression::{AsExpression, Expression};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Double, Nullable, SqlType};
use std::convert::TryFrom;
use std::io::Write;

use crate::Vector;

#[derive(SqlType, QueryId)]
#[diesel(postgres_type(name = "vector"))]
pub struct VectorType;

impl ToSql<VectorType, Pg> for Vector {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let dim = self.0.len();
        out.write_all(&u16::try_from(dim)?.to_be_bytes())?;
        out.write_all(&0_u16.to_be_bytes())?;

        for v in &self.0 {
            out.write_all(&v.to_be_bytes())?;
        }

        Ok(IsNull::No)
    }
}

impl FromSql<VectorType, Pg> for Vector {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        Vector::from_sql(value.as_bytes())
    }
}

diesel::infix_operator!(L2Distance, " <-> ", Double, backend: Pg);
diesel::infix_operator!(MaxInnerProduct, " <#> ", Double, backend: Pg);
diesel::infix_operator!(CosineDistance, " <=> ", Double, backend: Pg);

pub trait VectorExpressionMethods: Expression + Sized {
    fn l2_distance<T>(self, other: T) -> L2Distance<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Nullable<VectorType>>,
    {
        L2Distance::new(self, other.as_expression())
    }

    fn max_inner_product<T>(self, other: T) -> MaxInnerProduct<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Nullable<VectorType>>,
    {
        MaxInnerProduct::new(self, other.as_expression())
    }

    fn cosine_distance<T>(self, other: T) -> CosineDistance<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Nullable<VectorType>>,
    {
        CosineDistance::new(self, other.as_expression())
    }
}

impl<T: Expression> VectorExpressionMethods for T {}

#[cfg(test)]
mod tests {
    use crate::{Vector, VectorExpressionMethods};
    use diesel::pg::PgConnection;
    use diesel::{Connection, QueryDsl, RunQueryDsl};

    table! {
        use diesel::sql_types::*;

        items (id) {
            id -> Int4,
            embedding -> Nullable<crate::sql_types::Vector>,
        }
    }

    #[derive(Debug, Insertable, PartialEq, Queryable)]
    #[diesel(table_name = items)]
    struct Item {
        pub id: i32,
        pub embedding: Option<crate::Vector>,
    }

    #[test]
    fn it_works() -> Result<(), diesel::result::Error> {
        let mut conn = PgConnection::establish("postgres://localhost/pgvector_rust_test").unwrap();
        diesel::sql_query("CREATE EXTENSION IF NOT EXISTS vector").execute(&mut conn)?;
        diesel::sql_query("DROP TABLE IF EXISTS items").execute(&mut conn)?;
        diesel::sql_query("CREATE TABLE items (id serial PRIMARY KEY, embedding vector(3))")
            .execute(&mut conn)?;

        let new_items = vec![
            Item {
                id: 1,
                embedding: Some(Vector::from(vec![1.0, 1.0, 1.0])),
            },
            Item {
                id: 2,
                embedding: Some(Vector::from(vec![2.0, 2.0, 2.0])),
            },
            Item {
                id: 3,
                embedding: Some(Vector::from(vec![1.0, 1.0, 2.0])),
            },
            Item {
                id: 4,
                embedding: None,
            },
        ];

        diesel::insert_into(items::table)
            .values(&new_items)
            .get_results::<Item>(&mut conn)?;

        let all = items::table.load::<Item>(&mut conn)?;
        assert_eq!(4, all.len());

        let neighbors = items::table
            .order(items::embedding.l2_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)?;
        assert_eq!(
            vec![1, 3, 2, 4],
            neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>()
        );

        let neighbors = items::table
            .order(items::embedding.max_inner_product(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)?;
        assert_eq!(
            vec![2, 3, 1, 4],
            neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>()
        );

        let neighbors = items::table
            .order(items::embedding.cosine_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)?;
        assert_eq!(
            vec![1, 2, 3, 4],
            neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>()
        );

        let distances = items::table
            .select(items::embedding.max_inner_product(Vector::from(vec![1.0, 1.0, 1.0])))
            .order(items::id)
            .load::<Option<f64>>(&mut conn)?;
        assert_eq!(vec![Some(-3.0), Some(-6.0), Some(-4.0), None], distances);

        Ok(())
    }
}
