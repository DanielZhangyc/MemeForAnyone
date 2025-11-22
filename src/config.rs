use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub qdrant: QdrantConfig,
    pub ai: AiConfig,
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub backend: String, // "fs" or "s3"
    pub root: String,    // Local path or S3 bucket name
    // S3-specific fields
    pub s3_endpoint: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiConfig {
    pub active_embedding_model: String,
    pub active_rerank_model: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Local,
    Online,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModelUsage {
    Embedding,
    Rerank,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub r#type: ModelType,
    pub usage: ModelUsage,
    pub model_id: String,
    pub api_key_env: Option<String>,
    pub provider: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("storage.backend", "fs")?
            .set_default("storage.root", "./data")?
            .set_default("qdrant.url", "http://localhost:6334")?
            .set_default("qdrant.collection_name", "memes")?
            .set_default("ai.active_embedding_model", "bge_small")?
            .set_default("ai.active_rerank_model", "bge_reranker")?
            // Load from config files
            .add_source(File::with_name("config/models").required(true)) // Load models first
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            // Load from Environment variables
            .add_source(Environment::with_prefix("MFA").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
