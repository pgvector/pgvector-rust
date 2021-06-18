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
    #[test]
    fn it_works() {
    }
}
