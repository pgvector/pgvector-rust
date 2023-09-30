use std::cmp::PartialEq;
use std::convert::TryInto;
use std::error::Error;

#[cfg(feature = "diesel")]
use crate::diesel_ext::VectorType;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = VectorType))]
pub struct Vector(pub(crate) Vec<f32>);

impl From<Vec<f32>> for Vector {
    fn from(v: Vec<f32>) -> Self {
        Vector(v)
    }
}

impl Into<Vec<f32>> for Vector {
    fn into(self) -> Vec<f32> {
        self.0
    }
}

impl Vector {
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }

    pub(crate) fn from_sql(buf: &[u8]) -> Result<Vector, Box<dyn Error + Sync + Send>> {
        let dim = u16::from_be_bytes(buf[0..2].try_into()?) as usize;
        let unused = u16::from_be_bytes(buf[2..4].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut vec = Vec::with_capacity(dim);
        for i in 0..dim {
            let s = 4 + 4 * i;
            vec.push(f32::from_be_bytes(buf[s..s + 4].try_into()?));
        }

        Ok(Vector(vec))
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
}
