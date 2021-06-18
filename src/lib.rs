use byteorder::{BigEndian, ReadBytesExt};
use std::cmp::PartialEq;
use std::io::{Error, ErrorKind};

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

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
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn decode_vector(mut buf: &[u8]) -> std::io::Result<Vector> {
    let dim = buf.read_u16::<BigEndian>()?;
    let unused = buf.read_u16::<BigEndian>()?;
    if unused != 0 {
        return Err(Error::new(ErrorKind::Other, "expected unused to be 0"));
    }

    let mut vec = Vec::new();
    for _ in 0..dim {
        vec.push(buf.read_f32::<BigEndian>()?);
    }

    Ok(Vector(vec))
}

#[cfg(feature = "postgres")]
mod postgres_ext;

#[cfg(feature = "sqlx")]
mod sqlx_ext;

#[cfg(feature = "diesel")]
mod diesel_ext;

#[cfg(feature = "diesel")]
pub mod sql_types {
    pub use super::diesel_ext::VectorType as Vector;
}
