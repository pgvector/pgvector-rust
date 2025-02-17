use pgvector::Vector;
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
        "CREATE TABLE documents (id serial PRIMARY KEY, content text, embedding vector(1536))",
        &[],
    )?;

    let input = [
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ];
    let embeddings = embed(&input)?;
    for (content, embedding) in input.iter().zip(embeddings) {
        client.execute(
            "INSERT INTO documents (content, embedding) VALUES ($1, $2)",
            &[&content, &Vector::from(embedding)],
        )?;
    }

    let query = "forest";
    let query_embedding = embed(&[query])?.drain(..).next().unwrap();
    for row in client.query(
        "SELECT content FROM documents ORDER BY embedding <=> $1 LIMIT 5",
        &[&Vector::from(query_embedding)],
    )? {
        let content: &str = row.get(0);
        println!("{}", content);
    }

    Ok(())
}

fn embed(input: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").or(Err("Set OPENAI_API_KEY"))?;

    let response: Value = ureq::post("https://api.openai.com/v1/embeddings")
        .header("Authorization", &format!("Bearer {}", api_key))
        .send_json(serde_json::json!({
            "input": input,
            "model": "text-embedding-3-small",
        }))?
        .body_mut()
        .read_json()?;

    let embeddings = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v["embedding"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect()
        })
        .collect();

    Ok(embeddings)
}
