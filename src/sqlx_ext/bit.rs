use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use std::convert::TryFrom;

use crate::Bit;

impl<'a> Type<Postgres> for Bit<'a> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("bit")
    }
}

impl<'a> Encode<'a, Postgres> for Bit<'a> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let len = self.len;
        buf.extend(&i32::try_from(len).unwrap().to_be_bytes());

        for v in self.data {
            buf.extend(&v.to_be_bytes());
        }

        IsNull::No
    }
}

impl<'a> Decode<'a, Postgres> for Bit<'a> {
    fn decode(value: PgValueRef<'a>) -> Result<Self, BoxDynError> {
        let buf = <&[u8] as Decode<Postgres>>::decode(value)?;
        Bit::from_sql(buf)
    }
}

impl<'a> PgHasArrayType for Bit<'a> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_bit")
    }
}

#[cfg(test)]
mod tests {
    use crate::Bit;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::Row;

    #[async_std::test]
    async fn it_works() -> Result<(), sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://localhost/pgvector_rust_test")
            .await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await?;
        sqlx::query("DROP TABLE IF EXISTS sqlx_bit_items")
            .execute(&pool)
            .await?;
        sqlx::query("CREATE TABLE sqlx_bit_items (id bigserial PRIMARY KEY, embedding bit(8))")
            .execute(&pool)
            .await?;

        let vec = Bit::from_bytes(&[0b10101010]);
        let vec2 = Bit::from_bytes(&[0b01010101]);
        sqlx::query("INSERT INTO sqlx_bit_items (embedding) VALUES ($1), ($2), (NULL)")
            .bind(&vec)
            .bind(&vec2)
            .execute(&pool)
            .await?;

        let query_vec = Bit::from_bytes(&[0b10101010]);
        let row =
            sqlx::query("SELECT embedding FROM sqlx_bit_items ORDER BY embedding <~> $1 LIMIT 1")
                .bind(query_vec)
                .fetch_one(&pool)
                .await?;
        let res_vec: Bit = row.try_get("embedding").unwrap();
        assert_eq!(vec, res_vec);
        assert_eq!(&[0b10101010], res_vec.as_bytes());

        let null_row =
            sqlx::query("SELECT embedding FROM sqlx_bit_items WHERE embedding IS NULL LIMIT 1")
                .fetch_one(&pool)
                .await?;
        let null_res: Option<Bit> = null_row.try_get("embedding").unwrap();
        assert!(null_res.is_none());

        // ensures binary format is correct
        let text_row =
            sqlx::query("SELECT embedding::text FROM sqlx_bit_items ORDER BY id LIMIT 1")
                .fetch_one(&pool)
                .await?;
        let text_res: String = text_row.try_get("embedding").unwrap();
        assert_eq!("10101010", text_res);

        sqlx::query("ALTER TABLE sqlx_bit_items ADD COLUMN factors bit(8)[]")
            .execute(&pool)
            .await?;

        let vecs = &[vec, vec2];
        sqlx::query("INSERT INTO sqlx_bit_items (factors) VALUES ($1)")
            .bind(vecs)
            .execute(&pool)
            .await?;

        Ok(())
    }
}