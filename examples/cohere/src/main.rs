use pgvector::Bit;
use postgres::{Client, NoTls};
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_example")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;

    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;
    client.execute("DROP TABLE IF EXISTS documents", &[])?;
    client.execute(
        "CREATE TABLE documents (id serial PRIMARY KEY, content text, embedding bit(1536))",
        &[],
    )?;

    let input = [
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ];
    let embeddings = embed(&input, "search_document")?;
    for (content, embedding) in input.iter().zip(embeddings) {
        let embedding = Bit::from_bytes(&embedding);
        client.execute(
            "INSERT INTO documents (content, embedding) VALUES ($1, $2)",
            &[&content, &embedding],
        )?;
    }

    let query = "forest";
    let query_embedding = embed(&[query], "search_query")?;
    for row in client.query(
        "SELECT content FROM documents ORDER BY embedding <~> $1 LIMIT 5",
        &[&Bit::from_bytes(&query_embedding[0])],
    )? {
        let content: &str = row.get(0);
        println!("{}", content);
    }

    Ok(())
}

fn embed(texts: &[&str], input_type: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let api_key = std::env::var("CO_API_KEY").or(Err("Set CO_API_KEY"))?;

    let response: Value = ureq::post("https://api.cohere.com/v2/embed")
        .header("Authorization", &format!("Bearer {}", api_key))
        .send_json(serde_json::json!({
            "texts": texts,
            "model": "embed-v4.0",
            "input_type": input_type,
            "embedding_types": &["ubinary"],
        }))?
        .body_mut()
        .read_json()?;

    let embeddings = response["embeddings"]["ubinary"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap().try_into().unwrap())
                .collect()
        })
        .collect();

    Ok(embeddings)
}
