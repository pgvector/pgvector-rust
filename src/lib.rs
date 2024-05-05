#![doc = include_str!("../README.md")]

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

mod bit;
mod sparsevec;
mod vector;

pub use bit::Bit;
pub use sparsevec::SparseVec;
pub use vector::Vector;

#[cfg(feature = "halfvec")]
mod halfvec;

#[cfg(feature = "halfvec")]
pub use halfvec::HalfVec;

#[cfg(feature = "postgres")]
mod postgres_ext;

#[cfg(feature = "sqlx")]
mod sqlx_ext;

#[cfg(feature = "diesel")]
mod diesel_ext;

#[cfg(feature = "diesel")]
pub mod sql_types {
    pub use super::diesel_ext::halfvec::HalfVecType as HalfVec;
    pub use super::diesel_ext::vector::VectorType as Vector;
}

#[cfg(feature = "diesel")]
pub use diesel_ext::vector::VectorExpressionMethods;
