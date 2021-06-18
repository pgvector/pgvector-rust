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
    #[test]
    fn it_works() {
    }
}
