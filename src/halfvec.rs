use half::f16;

#[cfg(feature = "diesel")]
use crate::diesel_ext::halfvec::HalfVectorType;

#[cfg(feature = "diesel")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression};

/// A half vector.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = HalfVectorType))]
pub struct HalfVector(pub(crate) Vec<f16>);

impl From<Vec<f16>> for HalfVector {
    fn from(v: Vec<f16>) -> Self {
        HalfVector(v)
    }
}

impl From<HalfVector> for Vec<f16> {
    fn from(val: HalfVector) -> Self {
        val.0
    }
}

impl HalfVector {
    /// Creates a half vector from a `f32` slice.
    pub fn from_f32_slice(slice: &[f32]) -> HalfVector {
        HalfVector(slice.iter().map(|v| f16::from_f32(*v)).collect())
    }

    /// Returns a copy of the half vector as a `Vec<f16>`.
    pub fn to_vec(&self) -> Vec<f16> {
        self.0.clone()
    }

    /// Returns the half vector as a slice.
    pub fn as_slice(&self) -> &[f16] {
        self.0.as_slice()
    }

    #[cfg(any(feature = "postgres", feature = "sqlx", feature = "diesel"))]
    pub(crate) fn from_sql(
        buf: &[u8],
    ) -> Result<HalfVector, Box<dyn std::error::Error + Sync + Send>> {
        if buf.len() < 4 {
            return Err("invalid length".into());
        }

        let dim = u16::from_be_bytes(buf[0..2].try_into()?).into();
        let unused = u16::from_be_bytes(buf[2..4].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        if (buf.len() - 4) / 2 != dim || buf.len() % 2 != 0 {
            return Err("invalid length".into());
        }

        let mut vec = Vec::with_capacity(dim);
        for v in buf[4..].chunks_exact(2) {
            vec.push(f16::from_be_bytes(v.try_into()?));
        }

        Ok(HalfVector(vec))
    }
}

#[cfg(test)]
mod tests {
    use crate::HalfVector;
    use half::f16;

    #[test]
    fn test_into() {
        let vec = HalfVector::from(vec![
            f16::from_f32(1.0),
            f16::from_f32(2.0),
            f16::from_f32(3.0),
        ]);
        let f16_vec: Vec<f16> = vec.into();
        assert_eq!(
            f16_vec,
            vec![f16::from_f32(1.0), f16::from_f32(2.0), f16::from_f32(3.0)]
        );
    }

    #[test]
    fn test_to_vec() {
        let vec = HalfVector::from_f32_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(
            vec.to_vec(),
            vec![f16::from_f32(1.0), f16::from_f32(2.0), f16::from_f32(3.0)]
        );
    }

    #[test]
    fn test_as_slice() {
        let vec = HalfVector::from_f32_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(
            vec.as_slice(),
            &[f16::from_f32(1.0), f16::from_f32(2.0), f16::from_f32(3.0)]
        );
    }
}
