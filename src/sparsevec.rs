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
    pub(crate) indices: Vec<usize>,
    pub(crate) values: Vec<f32>,
}

impl SparseVector {
    /// Creates a sparse vector from a dense vector.
    pub fn from_dense(vec: &[f32]) -> SparseVector {
        let dim = vec.len();
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (i, v) in vec.iter().enumerate() {
            if *v != 0.0 {
                indices.push(i);
                values.push(*v);
            }
        }

        SparseVector {
            dim,
            indices,
            values,
        }
    }

    /// Creates a sparse vector from a map of non-zero elements.
    pub fn from_map<'a, I: IntoIterator<Item = (&'a usize, &'a f32)>>(
        map: I,
        dim: usize,
    ) -> SparseVector {
        let mut elements: Vec<(&usize, &f32)> = map.into_iter().filter(|v| *v.1 != 0.0).collect();
        elements.sort_by_key(|v| *v.0);
        let indices: Vec<usize> = elements.iter().map(|v| *v.0).collect();
        let values: Vec<f32> = elements.iter().map(|v| *v.1).collect();

        SparseVector {
            dim,
            indices,
            values,
        }
    }

    /// Returns the number of dimensions.
    pub fn dimensions(&self) -> usize {
        self.dim
    }

    /// Returns the non-zero indices.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns the non-zero values.
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// Returns the sparse vector as a `Vec<f32>`.
    pub fn to_vec(&self) -> Vec<f32> {
        let mut vec = vec![0.0; self.dim];
        for (i, v) in self.indices.iter().zip(&self.values) {
            vec[*i] = *v;
        }
        vec
    }

    #[cfg(any(feature = "postgres", feature = "sqlx", feature = "diesel"))]
    pub(crate) fn from_sql(
        buf: &[u8],
    ) -> Result<SparseVector, Box<dyn std::error::Error + Sync + Send>> {
        let dim = i32::from_be_bytes(buf[0..4].try_into()?).try_into()?;
        let nnz = i32::from_be_bytes(buf[4..8].try_into()?).try_into()?;
        let unused = i32::from_be_bytes(buf[8..12].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut indices = Vec::with_capacity(nnz);
        for i in 0..nnz {
            let s = 12 + 4 * i;
            indices.push(i32::from_be_bytes(buf[s..s + 4].try_into()?).try_into()?);
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
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_from_dense() {
        let vec = SparseVector::from_dense(&[1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
        assert_eq!(6, vec.dimensions());
        assert_eq!(&[0, 2, 4], vec.indices());
        assert_eq!(&[1.0, 2.0, 3.0], vec.values());
    }

    #[test]
    fn test_from_hash_map() {
        let map = HashMap::from([(0, 1.0), (2, 2.0), (4, 3.0)]);
        let vec = SparseVector::from_map(&map, 6);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
        assert_eq!(6, vec.dimensions());
        assert_eq!(&[0, 2, 4], vec.indices());
        assert_eq!(&[1.0, 2.0, 3.0], vec.values());
    }

    #[test]
    fn test_from_btree_map() {
        let map = BTreeMap::from([(0, 1.0), (2, 2.0), (4, 3.0)]);
        let vec = SparseVector::from_map(&map, 6);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
        assert_eq!(6, vec.dimensions());
        assert_eq!(&[0, 2, 4], vec.indices());
        assert_eq!(&[1.0, 2.0, 3.0], vec.values());
    }

    #[test]
    fn test_from_vec_map() {
        let vec = vec![(0, 1.0), (2, 2.0), (4, 3.0)];
        let map = vec.iter().map(|v| (&v.0, &v.1));
        let vec = SparseVector::from_map(map, 6);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0], vec.to_vec());
        assert_eq!(6, vec.dimensions());
        assert_eq!(&[0, 2, 4], vec.indices());
        assert_eq!(&[1.0, 2.0, 3.0], vec.values());
    }
}
