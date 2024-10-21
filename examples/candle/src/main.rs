// https://github.com/huggingface/candle/tree/main/candle-examples/examples/bert
// https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::api::sync::Api;
use pgvector::Vector;
use postgres::{Client, NoTls};
use std::error::Error;
use std::fs::read_to_string;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = Client::configure()
        .host("localhost")
        .dbname("pgvector_example")
        .user(std::env::var("USER")?.as_str())
        .connect(NoTls)?;

    client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;
    client.execute("DROP TABLE IF EXISTS documents", &[])?;
    client.execute(
        "CREATE TABLE documents (id serial PRIMARY KEY, content text, embedding vector(384))",
        &[],
    )?;

    let model = EmbeddingModel::new("sentence-transformers/all-MiniLM-L6-v2")?;

    let input = [
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ];
    let embeddings = input
        .iter()
        .map(|text| model.embed(text))
        .collect::<Result<Vec<_>, _>>()?;

    for (content, embedding) in input.iter().zip(embeddings) {
        client.execute(
            "INSERT INTO documents (content, embedding) VALUES ($1, $2)",
            &[&content, &Vector::from(embedding)],
        )?;
    }

    let document_id = 2;
    for row in client.query("SELECT content FROM documents WHERE id != $1 ORDER BY embedding <=> (SELECT embedding FROM documents WHERE id = $1) LIMIT 5", &[&document_id])? {
        let content: &str = row.get(0);
        println!("{}", content);
    }

    Ok(())
}

struct EmbeddingModel {
    tokenizer: Tokenizer,
    model: BertModel,
}

impl EmbeddingModel {
    pub fn new(model_id: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let api = Api::new()?;
        let repo = api.model(model_id.to_string());
        let tokenizer_path = repo.get("tokenizer.json")?;
        let config_path = repo.get("config.json")?;
        let weights_path = repo.get("model.safetensors")?;

        let mut tokenizer = Tokenizer::from_file(tokenizer_path)?;
        let padding = PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(padding));

        let device = Device::Cpu;
        let config: Config = serde_json::from_str(&read_to_string(config_path)?)?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self { tokenizer, model })
    }

    // TODO support multiple texts
    fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let tokens = self.tokenizer.encode(text, true)?;
        let token_ids = Tensor::new(vec![tokens.get_ids().to_vec()], &self.model.device)?;
        let token_type_ids = token_ids.zeros_like()?;
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;
        let embeddings = (embeddings.sum(1)? / (embeddings.dim(1)? as f64))?;
        let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;
        Ok(embeddings.squeeze(0)?.to_vec1::<f32>()?)
    }
}
