use pgvector::Vector;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::{Kind, Type};
use postgres::{Client, NoTls};
use rand::Rng;
use std::error::Error;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn Error>> {
    // generate random data
    let rows = 1000000;
    let dimensions = 128;
    let mut rng = rand::thread_rng();
    let embeddings: Vec<Vec<f32>> = (0..rows)
        .map(|_| (0..dimensions).map(|_| rng.gen()).collect())
        .collect();

    // enable extension
    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_example")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;
    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;

    // create table
    client.execute("DROP TABLE IF EXISTS items", &[])?;
    client.execute(
        &format!(
            "CREATE TABLE items (id bigserial, embedding vector({}))",
            dimensions
        ),
        &[],
    )?;

    // load data
    println!("Loading {} rows", embeddings.len());
    let vector_type = vector_type(&mut client)?;
    let writer = client.copy_in("COPY items (embedding) FROM STDIN WITH (FORMAT BINARY)")?;
    let mut writer = BinaryCopyInWriter::new(writer, &[vector_type]);
    for (i, embedding) in embeddings.into_iter().enumerate() {
        // show progress
        if i % 10000 == 0 {
            print!(".");
            io::stdout().flush()?;
        }

        writer.write(&[&Vector::from(embedding)])?;
    }
    writer.finish()?;
    println!("\nSuccess!");

    // create any indexes *after* loading initial data (skipping for this example)
    // println!("Creating index");
    // client.execute("SET maintenance_work_mem = '8GB'", &[])?;
    // client.execute("SET max_parallel_maintenance_workers = 7", &[])?;
    // client.execute("CREATE INDEX ON items USING hnsw (embedding vector_cosine_ops)", &[])?;

    // update planner statistics for good measure
    client.execute("ANALYZE items", &[])?;

    Ok(())
}

fn vector_type(client: &mut Client) -> Result<Type, Box<dyn Error>> {
    let row = client.query_one("SELECT pg_type.oid, nspname AS schema FROM pg_type INNER JOIN pg_namespace ON pg_namespace.oid = pg_type.typnamespace WHERE typname = 'vector'", &[])?;
    Ok(Type::new(
        "vector".into(),
        row.get("oid"),
        Kind::Simple,
        row.get("schema"),
    ))
}
