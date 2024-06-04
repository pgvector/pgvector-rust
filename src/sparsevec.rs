#[cfg(feature = "diesel")]
use crate::diesel_ext::sparsevec::SparseVectorType;

#[cfg(feature = "diesel")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression};

/// A sparse vector.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = SparseVectorType))]
pub struct SparseVector {
    pub(crate) dim: usize,
    pub(crate) indices: Vec<i32>,
    pub(crate) values: Vec<f32>,
}

impl SparseVector {
    /// Creates a sparse vector.
    pub fn new(dim: usize, indices: Vec<i32>, values: Vec<f32>) -> SparseVector {
        // TODO assert indices sorted
        assert_eq!(indices.len(), values.len());
        assert!(indices.len() < dim);

        SparseVector {
            dim,
            indices,
            values,
        }
    }

    /// Creates a sparse vector from a dense vector.
    pub fn from_dense(vec: &[f32]) -> SparseVector {
        let dim = vec.len();
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (i, v) in vec.iter().enumerate() {
            if *v != 0.0 {
                indices.push(i.try_into().unwrap());
                values.push(*v);
            }
        }

        SparseVector {
            dim,
            indices,
            values,
        }
    }

    /// Creates a sparse vector from coordinates.
    pub fn from_coordinates<I: IntoIterator<Item = (i32, f32)>>(
        iter: I,
        dim: usize,
    ) -> SparseVector {
        let mut elements: Vec<(i32, f32)> = iter.into_iter().collect();
        elements.sort_by_key(|v| v.0);
        let indices: Vec<i32> = elements.iter().map(|v| v.0).collect();
        let values: Vec<f32> = elements.iter().map(|v| v.1).collect();

        SparseVector {
            dim,
            indices,
            values,
        }
    }

    /// Returns the sparse vector as a `Vec<f32>`.
    pub fn to_vec(&self) -> Vec<f32> {
        let mut vec = vec![0.0; self.dim];
        for (i, v) in self.indices.iter().zip(&self.values) {
            vec[*i as usize] = *v;
        }
        vec
    }

    #[cfg(any(feature = "postgres", feature = "sqlx", feature = "diesel"))]
    pub(crate) fn from_sql(
        buf: &[u8],
    ) -> Result<SparseVector, Box<dyn std::error::Error + Sync + Send>> {
        let dim = i32::from_be_bytes(buf[0..4].try_into()?) as usize;
        let nnz = i32::from_be_bytes(buf[4..8].try_into()?) as usize;
        let unused = i32::from_be_bytes(buf[8..12].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut indices = Vec::with_capacity(nnz);
        for i in 0..nnz {
            let s = 12 + 4 * i;
            indices.push(i32::from_be_bytes(buf[s..s + 4].try_into()?));
        }

        let mut values = Vec::with_capacity(nnz);
        for i in 0..nnz {
            let s = 12 + 4 * nnz + 4 * i;
            values.push(f32::from_be_bytes(buf[s..s + 4].try_into()?));
        }

        Ok(SparseVector {
            dim,
            indices,
            values,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::SparseVector;
    use std::collections::HashMap;

    #[test]
    fn test_from_dense() {
        let vec = SparseVector::from_dense(&[1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
    }

    #[test]
    fn test_from_coordinates_map() {
        let elements = HashMap::from([(0, 1.0), (2, 2.0), (4, 3.0)]);
        let vec = SparseVector::from_coordinates(elements, 6);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
    }

    #[test]
    fn test_from_coordinates_vec() {
        let elements = vec![(0, 1.0), (2, 2.0), (4, 3.0)];
        let vec = SparseVector::from_coordinates(elements, 6);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
    }

    #[test]
    fn test_to_vec() {
        let vec = SparseVector::new(6, vec![0, 2, 4], vec![1.0, 2.0, 3.0]);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
    }
}
