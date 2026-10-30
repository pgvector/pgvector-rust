#[cfg(any(feature = "postgres", feature = "sqlx", feature = "diesel"))]
use std::convert::TryInto;

#[cfg(feature = "diesel")]
use crate::diesel_ext::vector::VectorType;

#[cfg(feature = "diesel")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A vector.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = VectorType))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Vector(pub(crate) Vec<f32>);

impl From<Vec<f32>> for Vector {
    fn from(v: Vec<f32>) -> Self {
        Vector(v)
    }
}

impl From<Vector> for Vec<f32> {
    fn from(val: Vector) -> Self {
        val.0
    }
}

impl Vector {
    /// Returns a copy of the vector as a `Vec<f32>`.
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }

    /// Returns the vector as a slice.
    pub fn as_slice(&self) -> &[f32] {
        self.0.as_slice()
    }

    #[cfg(any(feature = "postgres", feature = "sqlx", feature = "diesel", test))]
    pub(crate) fn from_sql(buf: &[u8]) -> Result<Vector, Box<dyn std::error::Error + Sync + Send>> {
        if buf.len() < 4 {
            return Err("invalid length".into());
        }

        let dim = u16::from_be_bytes(buf[0..2].try_into()?).into();
        let unused = u16::from_be_bytes(buf[2..4].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        if (buf.len() - 4) / 4 != dim || buf.len() % 4 != 0 {
            return Err("invalid length".into());
        }

        let mut vec = Vec::with_capacity(dim);
        for v in buf[4..].chunks_exact(4) {
            vec.push(f32::from_be_bytes(v.try_into()?));
        }

        Ok(Vector(vec))
    }
}

#[cfg(test)]
mod tests {
    use crate::Vector;

    #[test]
    fn test_into() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let f32_vec: Vec<f32> = vec.into();
        assert_eq!(f32_vec, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_to_vec() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.to_vec(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_as_slice() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() {
        let vec = Vector::from(vec![1.0, 2.0, 3.0]);
        let json = serde_json::to_string(&vec).unwrap();
        assert_eq!(json, "[1.0,2.0,3.0]");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        let json = "[1.0,2.0,3.0]";
        let vec: Vector = serde_json::from_str(json).unwrap();
        assert_eq!(vec, Vector::from(vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_from_sql() {
        let mut buf = Vec::new();
        buf.extend(3_i16.to_be_bytes());
        buf.extend(0_i16.to_be_bytes());
        buf.extend(1.0_f32.to_be_bytes());
        buf.extend(2.0_f32.to_be_bytes());
        buf.extend(3.0_f32.to_be_bytes());
        assert_eq!(
            Vector::from(vec![1.0, 2.0, 3.0]),
            Vector::from_sql(&buf).unwrap()
        );
    }

    #[test]
    fn test_from_sql_zero_length() {
        assert_eq!(
            "invalid length",
            Vector::from_sql(&[]).unwrap_err().to_string()
        );
    }

    #[test]
    fn test_from_sql_missing_data() {
        let mut buf = Vec::new();
        buf.extend(2_i16.to_be_bytes());
        buf.extend(0_i16.to_be_bytes());
        buf.extend(1.0_f32.to_be_bytes());
        assert_eq!(
            "invalid length",
            Vector::from_sql(&buf).unwrap_err().to_string()
        );
    }

    #[test]
    fn test_from_sql_extra_data() {
        let mut buf = Vec::new();
        buf.extend(2_i16.to_be_bytes());
        buf.extend(0_i16.to_be_bytes());
        buf.extend(1.0_f32.to_be_bytes());
        buf.extend(2.0_f32.to_be_bytes());
        buf.extend(3.0_f32.to_be_bytes());
        assert_eq!(
            "invalid length",
            Vector::from_sql(&buf).unwrap_err().to_string()
        );
    }

    #[test]
    fn test_from_sql_non_zero_unused() {
        let mut buf = Vec::new();
        buf.extend(0_i16.to_be_bytes());
        buf.extend(1_i16.to_be_bytes());
        assert_eq!(
            "expected unused to be 0",
            Vector::from_sql(&buf).unwrap_err().to_string()
        );
    }
}
