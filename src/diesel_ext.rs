use byteorder::{BigEndian, WriteBytesExt};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::convert::TryInto;
use std::io::Write;

use crate::Vector;

#[derive(SqlType)]
#[postgres(type_name = "Vector")]
pub struct VectorType;

impl ToSql<VectorType, Pg> for Vector {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let dim = self.0.len();
        if dim > 1024 {
            return Err("vector cannot have more than 1024 dimensions".into())
        }
        if dim < 1 {
            return Err("vector must have at least 1 dimension".into())
        }

        out.write_u16::<BigEndian>(dim.try_into()?)?;
        out.write_u16::<BigEndian>(0)?;
        for v in self.0.iter() {
            out.write_f32::<BigEndian>(*v)?;
        }

        Ok(IsNull::No)
    }
}

impl FromSql<VectorType, Pg> for Vector {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let buf = not_none!(bytes);
        crate::decode_vector(buf).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
