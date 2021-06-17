use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
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
        let mut buf = not_none!(bytes);
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
