use pgvector::Vector;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::{Kind, Type};
use postgres::{Client, NoTls};
use rand::Rng;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // generate random data
    let rows = 100000;
    let dimensions = 128;
    let mut rng = rand::rng();
    let embeddings: Vec<Vec<f32>> = (0..rows)
        .map(|_| (0..dimensions).map(|_| rng.random()).collect())
        .collect();
    let categories: Vec<i64> = (0..rows).map(|_| rng.random_range(1..=100)).collect();
    let queries: Vec<Vec<f32>> = (0..10)
        .map(|_| (0..dimensions).map(|_| rng.random()).collect())
        .collect();

    // enable extensions
    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_citus")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;
    client.execute("CREATE EXTENSION IF NOT EXISTS citus", &[])?;
    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;

    // GUC variables set on the session do not propagate to Citus workers
    // https://github.com/citusdata/citus/issues/462
    // you can either:
    // 1. set them on the system, user, or database and reconnect
    // 2. set them for a transaction with SET LOCAL
    client.execute(
        "ALTER DATABASE pgvector_citus SET maintenance_work_mem = '512MB'",
        &[],
    )?;
    client.execute("ALTER DATABASE pgvector_citus SET hnsw.ef_search = 20", &[])?;
    client.close()?;

    // reconnect for updated GUC variables to take effect
    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_citus")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;

    println!("Creating distributed table");
    client.execute("DROP TABLE IF EXISTS items", &[])?;
    client.execute(
        &format!("CREATE TABLE items (id bigserial, embedding vector({dimensions}), category_id bigint, PRIMARY KEY (id, category_id))"),
        &[],
    )?;
    client.execute("SET citus.shard_count = 4", &[])?;
    client.execute(
        "SELECT create_distributed_table('items', 'category_id')",
        &[],
    )?;

    println!("Loading data in parallel");
    let vector_type = get_type(&mut client, "vector")?;
    let writer =
        client.copy_in("COPY items (embedding, category_id) FROM STDIN WITH (FORMAT BINARY)")?;
    let mut writer = BinaryCopyInWriter::new(writer, &[vector_type, Type::INT8]);
    for (embedding, category) in embeddings.into_iter().zip(categories) {
        writer.write(&[&Vector::from(embedding), &category])?;
    }
    writer.finish()?;

    println!("Creating index in parallel");
    client.execute(
        "CREATE INDEX ON items USING hnsw (embedding vector_l2_ops)",
        &[],
    )?;

    println!("Running distributed queries");
    for query in queries {
        let rows = client.query(
            "SELECT id FROM items ORDER BY embedding <-> $1 LIMIT 10",
            &[&Vector::from(query)],
        )?;
        let ids: Vec<i64> = rows.into_iter().map(|row| row.get(0)).collect();
        println!("{:?}", ids);
    }

    Ok(())
}

fn get_type(client: &mut Client, name: &str) -> Result<Type, Box<dyn Error>> {
    let row = client.query_one("SELECT pg_type.oid, nspname AS schema FROM pg_type INNER JOIN pg_namespace ON pg_namespace.oid = pg_type.typnamespace WHERE typname = $1", &[&name])?;
    Ok(Type::new(
        name.into(),
        row.get("oid"),
        Kind::Simple,
        row.get("schema"),
    ))
}
