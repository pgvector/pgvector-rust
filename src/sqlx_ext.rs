use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use sqlx::{Decode, Encode, Postgres, Type};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use std::convert::TryInto;

use crate::Vector;

impl Type<Postgres> for Vector {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("vector")
    }
}

impl Encode<'_, Postgres> for Vector {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut w = BytesMut::new();
        let dim = self.0.len();

        w.put_u16(dim.try_into().unwrap());
        w.put_u16(0);
        for v in self.0.iter() {
            w.put_f32(*v);
        }

        buf.extend(&w[..]);

        IsNull::No
    }
}

impl Decode<'_, Postgres> for Vector {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let mut buf = <&[u8] as Decode<Postgres>>::decode(value)?;
        let dim = buf.read_u16::<BigEndian>()?;
        let unused = buf.read_u16::<BigEndian>()?;
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut vec = Vec::new();
        for _ in 0..dim {
            vec.push(buf.read_f32::<BigEndian>()?);
        }

        Ok(Vector(vec))
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
        sqlx::query("DROP TABLE IF EXISTS t").execute(&pool).await?;
        sqlx::query("CREATE TABLE t (id bigserial primary key, c vector(3))").execute(&pool).await?;

        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let vec2 = Vector::from(vec![4.0, 5.0, 6.0]);
        sqlx::query("INSERT INTO t (c) VALUES ($1), ($2), (NULL)").bind(&vec).bind(&vec2).execute(&pool).await?;

        let query_vec = Vector::from(vec![3.0, 1.0, 2.0]);
        let row = sqlx::query("SELECT c from t ORDER BY c <-> $1 LIMIT 1").bind(query_vec).fetch_one(&pool).await?;
        let res_vec: Vector = row.try_get("c").unwrap();
        assert_eq!(vec, res_vec);
        assert_eq!(vec![1.0, 2.0, 3.0], res_vec.to_vec());

        Ok(())
    }
}
