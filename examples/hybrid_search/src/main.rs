// https://github.com/huggingface/candle/tree/main/candle-examples/examples/bert
// https://huggingface.co/sentence-transformers/multi-qa-MiniLM-L6-cos-v1

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
    client.execute(
        "CREATE INDEX ON documents USING GIN (to_tsvector('english', content))",
        &[],
    )?;

    let model = EmbeddingModel::new("sentence-transformers/multi-qa-MiniLM-L6-cos-v1")?;

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

    let query = "growling bear";
    let query_embedding = model.embed(query)?;
    let k = 60.0;

    for row in client.query(HYBRID_SQL, &[&query, &Vector::from(query_embedding), &k])? {
        let id: i32 = row.get(0);
        let score: f64 = row.get(1);
        println!("document: {}, RRF score: {}", id, score);
    }

    Ok(())
}

const HYBRID_SQL: &str = "
WITH semantic_search AS (
    SELECT id, RANK () OVER (ORDER BY embedding <=> $2) AS rank
    FROM documents
    ORDER BY embedding <=> $2
    LIMIT 20
),
keyword_search AS (
    SELECT id, RANK () OVER (ORDER BY ts_rank_cd(to_tsvector('english', content), query) DESC)
    FROM documents, plainto_tsquery('english', $1) query
    WHERE to_tsvector('english', content) @@ query
    ORDER BY ts_rank_cd(to_tsvector('english', content), query) DESC
    LIMIT 20
)
SELECT
    COALESCE(semantic_search.id, keyword_search.id) AS id,
    COALESCE(1.0 / ($3::double precision + semantic_search.rank), 0.0) +
    COALESCE(1.0 / ($3::double precision + keyword_search.rank), 0.0) AS score
FROM semantic_search
FULL OUTER JOIN keyword_search ON semantic_search.id = keyword_search.id
ORDER BY score DESC
LIMIT 5
";

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
