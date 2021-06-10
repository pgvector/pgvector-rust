# pgvector.rs

[pgvector](https://github.com/ankane/pgvector) support for Rust

Supports the [postgres](https://github.com/sfackler/rust-postgres) crate

[![Build Status](https://github.com/ankane/pgvector.rs/workflows/build/badge.svg?branch=master)](https://github.com/ankane/pgvector.rs/actions)

## Installation

Add this line your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = "0.1"
```

## Getting Started

Create a vector from a `Vec<f32>`

```rust
let vec = pgvector::Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
client.execute("INSERT INTO table (column) VALUES ($1)", &[&vec])?;
```

Get the nearest neighbor

```rust
let row = client.query_one("SELECT * FROM table ORDER BY column <-> $1 LIMIT 1", &[&vec])?;
```

Retrieve a vector

```rust
let row = client.query_one("SELECT column FROM table LIMIT 1", &[])?;
let vec: pgvector::Vector = row.get(0);
```

Use `Option` if the value could be `NULL`

```rust
let res: Option<pgvector::Vector> = row.get(0);
```

Convert a vector to a `Vec<f32>`

```rust
let f32_vec = vec.to_vec();
```

## History

View the [changelog](https://github.com/ankane/pgvector.rs/blob/master/CHANGELOG.md)

## Contributing

Everyone is encouraged to help improve this project. Here are a few ways you can help:

- [Report bugs](https://github.com/ankane/pgvector.rs/issues)
- Fix bugs and [submit pull requests](https://github.com/ankane/pgvector.rs/pulls)
- Write, clarify, or fix documentation
- Suggest or add new features

To get started with development:

```sh
git clone https://github.com/ankane/pgvector.rs.git
cd pgvector.rs
cargo test
```
