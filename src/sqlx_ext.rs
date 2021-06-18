use bytes::{BytesMut};
use sqlx::{Decode, Encode, Postgres, Type};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};

use crate::Vector;

impl Type<Postgres> for Vector {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("vector")
    }
}

impl Encode<'_, Postgres> for Vector {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut w = BytesMut::new();
        self.to_sql(&mut w).unwrap();
        buf.extend(&w[..]);
        IsNull::No
    }
}

impl Decode<'_, Postgres> for Vector {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let buf = <&[u8] as Decode<Postgres>>::decode(value)?;
        Vector::from_sql(buf).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    #[async_std::test]
    async fn it_works() -> Result<(), sqlx::Error> {
        use crate::Vector;
        use sqlx::postgres::PgPoolOptions;
        use sqlx::Row;

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://localhost/pgvector_rust_test").await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector").execute(&pool).await?;
        sqlx::query("DROP TABLE IF EXISTS t2").execute(&pool).await?;
        sqlx::query("CREATE TABLE t2 (id bigserial primary key, c vector(3))").execute(&pool).await?;

        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let vec2 = Vector::from(vec![4.0, 5.0, 6.0]);
        sqlx::query("INSERT INTO t2 (c) VALUES ($1), ($2), (NULL)").bind(&vec).bind(&vec2).execute(&pool).await?;

        let query_vec = Vector::from(vec![3.0, 1.0, 2.0]);
        let row = sqlx::query("SELECT c from t2 ORDER BY c <-> $1 LIMIT 1").bind(query_vec).fetch_one(&pool).await?;
        let res_vec: Vector = row.try_get("c").unwrap();
        assert_eq!(vec, res_vec);
        assert_eq!(vec![1.0, 2.0, 3.0], res_vec.to_vec());

        let empty_vec = Vector::from(vec![]);
        let empty_res = sqlx::query("INSERT INTO t (c) VALUES ($1)").bind(&empty_vec).execute(&pool).await;
        assert!(empty_res.is_err());
        assert!(empty_res.unwrap_err().to_string().contains("vector must have at least 1 dimension"));

        Ok(())
    }
}
