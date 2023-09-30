#![doc = include_str!("../README.md")]

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

mod vector;
pub use vector::Vector;

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

#[cfg(feature = "diesel")]
pub use diesel_ext::VectorExpressionMethods;
