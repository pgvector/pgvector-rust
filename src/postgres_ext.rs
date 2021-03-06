use bytes::BytesMut;
use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::error::Error;

use crate::Vector;

impl<'a> FromSql<'a> for Vector {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Vector, Box<dyn Error + Sync + Send>> {
        Vector::from_sql(raw)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "vector"
    }
}

impl ToSql for Vector {
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(w)?;
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "vector"
    }

    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::Vector;
        use postgres::config::Config;
        use postgres::NoTls;

        let user = std::env::var("USER").unwrap();
        let mut client = Config::new().host("localhost").dbname("pgvector_rust_test").user(user.as_str()).connect(NoTls).unwrap();

        client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[]).unwrap();
        client.execute("DROP TABLE IF EXISTS t", &[]).unwrap();
        client.execute("CREATE TABLE t (id bigserial primary key, c vector(3))", &[]).unwrap();

        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let vec2 = Vector::from(vec![4.0, 5.0, 6.0]);
        client.execute("INSERT INTO t (c) VALUES ($1), ($2), (NULL)", &[&vec, &vec2]).unwrap();

        let query_vec = Vector::from(vec![3.0, 1.0, 2.0]);
        let row = client.query_one("SELECT c from t ORDER BY c <-> $1 LIMIT 1", &[&query_vec]).unwrap();
        let res_vec: Vector = row.get(0);
        assert_eq!(vec, res_vec);
        assert_eq!(vec![1.0, 2.0, 3.0], res_vec.to_vec());

        let empty_vec = Vector::from(vec![]);
        let empty_res = client.execute("INSERT INTO t (c) VALUES ($1)", &[&empty_vec]);
        assert!(empty_res.is_err());
        assert!(empty_res.unwrap_err().to_string().contains("vector must have at least 1 dimension"));

        let null_row = client.query_one("SELECT c from t WHERE c IS NULL LIMIT 1", &[]).unwrap();
        let null_res: Option<Vector> = null_row.get(0);
        assert!(null_res.is_none());

        // ensures binary format is correct
        let text_row = client.query_one("SELECT c::text from t ORDER BY id LIMIT 1", &[]).unwrap();
        let text_res: String = text_row.get(0);
        assert_eq!("[1,2,3]", text_res);
    }
}
