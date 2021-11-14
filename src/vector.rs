use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use std::cmp::PartialEq;
use std::convert::TryInto;
use std::error::Error;

#[cfg(feature = "diesel")]
use crate::diesel_ext::VectorType;

#[derive(Debug)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "diesel", sql_type = "VectorType")]
pub struct Vector(Vec<f32>);

impl From<Vec<f32>> for Vector {
    fn from(v: Vec<f32>) -> Self {
        Vector(v)
    }
}

impl Vector {
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }

    pub(crate) fn from_sql(mut buf: &[u8]) -> Result<Vector, Box<dyn Error + Sync + Send>> {
        let dim = buf.read_u16::<BigEndian>()?;
        let unused = buf.read_u16::<BigEndian>()?;
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut vec = vec![0.0; dim as usize];
        buf.read_f32_into::<BigEndian>(&mut vec)?;

        Ok(Vector(vec))
    }

    pub(crate) fn to_sql(&self, w: &mut BytesMut) -> Result<(), Box<dyn Error + Sync + Send>> {
        let dim = self.0.len();
        w.put_u16(dim.try_into()?);
        w.put_u16(0);

        for v in self.0.iter() {
            w.put_f32(*v);
        }

        Ok(())
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
