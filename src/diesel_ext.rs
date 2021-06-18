use bytes::{BytesMut};
use diesel::deserialize::{self, FromSql};
use diesel::expression::{AsExpression, Expression};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

use crate::Vector;

#[derive(SqlType, QueryId)]
#[postgres(type_name = "Vector")]
pub struct VectorType;

impl ToSql<VectorType, Pg> for Vector {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let mut w = BytesMut::new();
        self.to_sql(&mut w).unwrap();
        out.write_all(&w)?;
        Ok(IsNull::No)
    }
}

impl FromSql<VectorType, Pg> for Vector {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let buf = not_none!(bytes);
        Vector::from_sql(buf).map_err(|e| e.into())
    }
}

diesel_infix_operator!(L2Distance, " <-> ", backend: Pg);
diesel_infix_operator!(MaxInnerProduct, " <#> ", backend: Pg);
diesel_infix_operator!(CosineDistance, " <=> ", backend: Pg);

// don't specify a SqlType since it won't work with Nullable<Vector>
pub trait VectorExpressionMethods: Expression + Sized {
    fn l2_distance<T: AsExpression<Self::SqlType>>(self, other: T) -> L2Distance<Self, T::Expression> {
        L2Distance::new(self, other.as_expression())
    }

    fn max_inner_product<T: AsExpression<Self::SqlType>>(self, other: T) -> MaxInnerProduct<Self, T::Expression> {
        MaxInnerProduct::new(self, other.as_expression())
    }

    fn cosine_distance<T: AsExpression<Self::SqlType>>(self, other: T) -> CosineDistance<Self, T::Expression> {
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
    #[table_name="items"]
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

        let conn = PgConnection::establish("postgres://localhost/pgvector_rust_test").unwrap();
        conn.execute("CREATE EXTENSION IF NOT EXISTS vector").unwrap();
        conn.execute("DROP TABLE IF EXISTS items").unwrap();
        conn.execute("CREATE TABLE items (id serial primary key, factors vector(3))").unwrap();

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

        diesel::insert_into(items::table).values(&new_items).get_results::<Item>(&conn).unwrap();

        let all = items::table.load::<Item>(&conn).unwrap();
        assert_eq!(3, all.len());

        let neighbors = items::table
            .order(items::factors.l2_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&conn)
            .unwrap();
        assert_eq!(vec![1, 3, 2], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());

        let neighbors = items::table
            .order(items::factors.max_inner_product(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&conn)
            .unwrap();
        assert_eq!(vec![2, 3, 1], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());

        let neighbors = items::table
            .order(items::factors.cosine_distance(Vector::from(vec![1.0, 1.0, 1.0])))
            .limit(5)
            .load::<Item>(&conn)
            .unwrap();
        assert_eq!(vec![1, 2, 3], neighbors.into_iter().map(|v| v.id).collect::<Vec<i32>>());
    }
}
