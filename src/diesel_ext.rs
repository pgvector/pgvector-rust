use bytes::BytesMut;
use diesel::backend;
use diesel::deserialize::{self, FromSql};
use diesel::expression::{AsExpression, Expression};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Double, SqlType};
use std::io::Write;

use crate::Vector;

#[derive(SqlType, QueryId)]
#[diesel(postgres_type(name = "vector"))]
pub struct VectorType;

impl ToSql<VectorType, Pg> for Vector {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let mut w = BytesMut::new();
        self.to_sql(&mut w)?;
        out.write_all(&w)?;
        Ok(IsNull::No)
    }
}

impl FromSql<VectorType, Pg> for Vector {
    fn from_sql(value: backend::RawValue<'_, Pg>) -> deserialize::Result<Self> {
        Vector::from_sql(value.as_bytes())
    }
}

diesel::infix_operator!(L2Distance, " <-> ", Double, backend: Pg);
diesel::infix_operator!(MaxInnerProduct, " <#> ", Double, backend: Pg);
diesel::infix_operator!(CosineDistance, " <=> ", Double, backend: Pg);

// don't specify a SqlType since it won't work with Nullable<Vector>
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
}

impl<T: Expression> VectorExpressionMethods for T {}

#[cfg(test)]
mod tests {
    table! {
        use diesel::sql_types::*;

        items (id) {
            id -> Int4,
            factors -> Nullable<crate::sql_types::Vector>,
        }
    }

    #[derive(Debug, Insertable, PartialEq, Queryable)]
    #[diesel(table_name = items)]
    struct Item {
        pub id: i32,
        pub factors: Option<crate::Vector>
    }

    #[test]
    fn it_works() {
        use crate::Vector;
        use crate::VectorExpressionMethods;
        use diesel::pg::PgConnection;
        use diesel::{Connection, QueryDsl, RunQueryDsl};

        let mut conn = PgConnection::establish("postgres://localhost/pgvector_rust_test").unwrap();
        diesel::sql_query("CREATE EXTENSION IF NOT EXISTS vector").execute(&mut conn).unwrap();
        diesel::sql_query("DROP TABLE IF EXISTS items").execute(&mut conn).unwrap();
        diesel::sql_query("CREATE TABLE items (id serial primary key, factors vector(3))").execute(&mut conn).unwrap();

        let new_items = vec![
            Item {
                id: 1,
                factors: Some(Vector::from(vec![1.0, 1.0, 1.0]))
            },
            Item {
                id: 2,
                factors: Some(Vector::from(vec![2.0, 2.0, 2.0]))
            },
            Item {
                id: 3,
                factors: Some(Vector::from(vec![1.0, 1.0, 2.0]))
            },
        ];

        diesel::insert_into(items::table).values(&new_items).get_results::<Item>(&mut conn).unwrap();

        let all = items::table.load::<Item>(&mut conn).unwrap();
        assert_eq!(3, all.len());

        let neighbors = items::table
            .order(items::factors.l2_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)
            .unwrap();
        assert_eq!(vec![1, 3, 2], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());

        let neighbors = items::table
            .order(items::factors.max_inner_product(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)
            .unwrap();
        assert_eq!(vec![2, 3, 1], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());

        let neighbors = items::table
            .order(items::factors.cosine_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&mut conn)
            .unwrap();
        assert_eq!(vec![1, 2, 3], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());

        let distances = items::table
            .select(items::factors.max_inner_product(Vector::from(vec![1.0, 1.0, 1.0])))
            .order(items::id)
            .load::<Option<f64>>(&mut conn)
            .unwrap();
        assert_eq!(vec![Some(-3.0), Some(-6.0), Some(-4.0)], distances);
    }
}
