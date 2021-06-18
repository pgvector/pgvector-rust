use bytes::{BytesMut};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

use crate::Vector;

#[derive(SqlType)]
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
        use diesel::pg::PgConnection;
        use diesel::Connection;
        use diesel::RunQueryDsl;

        let conn = PgConnection::establish("postgres://localhost/pgvector_rust_test").unwrap();
        conn.execute("CREATE EXTENSION IF NOT EXISTS vector").unwrap();
        conn.execute("DROP TABLE IF EXISTS items").unwrap();
        conn.execute("CREATE TABLE items (id serial primary key, factors vector(3))").unwrap();

        let factors = Vector::from(vec![1.0, 2.0, 3.0]);
        let new_item = Item {
            id: 1,
            factors: Some(factors)
        };

        let item: Item = diesel::insert_into(items::table)
            .values(&new_item)
            .get_result(&conn)
            .unwrap();

        assert_eq!(new_item, item);
    }
}
