# pgvector-rust

[pgvector](https://github.com/pgvector/pgvector) support for Rust

Supports [Rust-Postgres](https://github.com/sfackler/rust-postgres), [SQLx](https://github.com/launchbadge/sqlx), and [Diesel](https://github.com/diesel-rs/diesel)

[![Build Status](https://github.com/pgvector/pgvector-rust/workflows/build/badge.svg?branch=master)](https://github.com/pgvector/pgvector-rust/actions)

## Getting Started

Follow the instructions for your database library:

- [Rust-Postgres](#rust-postgres)
- [SQLx](#sqlx)
- [Diesel](#diesel)

Or check out some examples:

- [Embeddings](https://github.com/pgvector/pgvector-rust/blob/master/examples/openai/src/main.rs) with OpenAI
- [Recommendations](https://github.com/pgvector/pgvector-rust/blob/master/examples/disco/src/main.rs) with Disco

## Rust-Postgres

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.3", features = ["postgres"] }
```

Enable the extension

```rust
client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;
```

Create a table

```rust
client.execute("CREATE TABLE items (id bigserial PRIMARY KEY, embedding vector(3))", &[])?;
```

Create a vector from a `Vec<f32>`

```rust
use pgvector::Vector;

let embedding = Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
client.execute("INSERT INTO items (embedding) VALUES ($1)", &[&embedding])?;
```

Get the nearest neighbor

```rust
let row = client.query_one(
    "SELECT * FROM items ORDER BY embedding <-> $1 LIMIT 1",
    &[&embedding],
)?;
```

Retrieve a vector

```rust
let row = client.query_one("SELECT embedding FROM items LIMIT 1", &[])?;
let embedding: Vector = row.get(0);
```

Use `Option` if the value could be `NULL`

```rust
let embedding: Option<Vector> = row.get(0);
```

## SQLx

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.3", features = ["sqlx"] }
```

Enable the extension

```rust
sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
    .execute(&pool)
    .await?;
```

Create a table

```rust
sqlx::query("CREATE TABLE items (id bigserial PRIMARY KEY, embedding vector(3))")
    .execute(&pool)
    .await?;
```

Create a vector from a `Vec<f32>`

```rust
use pgvector::Vector;

let embedding = Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
sqlx::query("INSERT INTO items (embedding) VALUES ($1)")
    .bind(embedding)
    .execute(&pool)
    .await?;
```

Get the nearest neighbors

```rust
let rows = sqlx::query("SELECT * FROM items ORDER BY embedding <-> $1 LIMIT 1")
    .bind(embedding)
    .fetch_all(&pool)
    .await?;
```

Retrieve a vector

```rust
let row = sqlx::query("SELECT embedding FROM items LIMIT 1").fetch_one(&pool).await?;
let embedding: Vector = row.try_get("embedding")?;
```

## Diesel

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.3", features = ["diesel"] }
```

And add this line to your application’s `diesel.toml` under `[print_schema]`:

```toml
import_types = ["diesel::sql_types::*", "pgvector::sql_types::*"]
```

Create a migration

```sh
diesel migration generate create_vector_extension
```

with `up.sql`:

```sql
CREATE EXTENSION vector
```

and `down.sql`:

```sql
DROP EXTENSION vector
```

Run the migration

```sql
diesel migration run
```

You can now use the `vector` type in future migrations

```sql
CREATE TABLE items (
  id SERIAL PRIMARY KEY,
  embedding VECTOR(3)
)
```

For models, use:

```rust
use pgvector::Vector;

#[derive(Queryable)]
#[diesel(table_name = items)]
pub struct Item {
    pub id: i32,
    pub embedding: Option<Vector>,
}

#[derive(Insertable)]
#[diesel(table_name = items)]
pub struct NewItem {
    pub embedding: Option<Vector>,
}
```

Create a vector from a `Vec<f32>`

```rust
let embedding = Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
let new_item = NewItem {
    embedding: Some(embedding)
};

diesel::insert_into(items::table)
    .values(&new_item)
    .get_result::<Item>(&mut conn)?;
```

Get the nearest neighbors

```rust
use pgvector::VectorExpressionMethods;

let neighbors = items::table
    .order(items::embedding.l2_distance(embedding))
    .limit(5)
    .load::<Item>(&mut conn)?;
```

Also supports `max_inner_product` and `cosine_distance`

Get the distances

```rust
let distances = items::table
    .select(items::embedding.l2_distance(embedding))
    .load::<Option<f64>>(&mut conn)?;
```

Add an approximate index in a migration

```sql
CREATE INDEX my_index ON items USING ivfflat (embedding vector_l2_ops) WITH (lists = 100)
-- or
CREATE INDEX my_index ON items USING hnsw (embedding vector_l2_ops)
```

Use `vector_ip_ops` for inner product and `vector_cosine_ops` for cosine distance

## Serialization

Use the `serde` feature to enable serialization

## Reference

Convert a vector to a `Vec<f32>`

```rust
let f32_vec: Vec<f32> = vec.into();
```

Get a slice [unreleased]

```rust
let slice = vec.as_slice();
```

## History

View the [changelog](https://github.com/pgvector/pgvector-rust/blob/master/CHANGELOG.md)

## Contributing

Everyone is encouraged to help improve this project. Here are a few ways you can help:

- [Report bugs](https://github.com/pgvector/pgvector-rust/issues)
- Fix bugs and [submit pull requests](https://github.com/pgvector/pgvector-rust/pulls)
- Write, clarify, or fix documentation
- Suggest or add new features

To get started with development:

```sh
git clone https://github.com/pgvector/pgvector-rust.git
cd pgvector-rust
createdb pgvector_rust_test
cargo test --all-features
```
