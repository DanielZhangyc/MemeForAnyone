mod config;
mod storage;

use config::AppConfig;
use anyhow::Context;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::load()?;
    info!("Configuration loaded successfully");
    info!("Server: {}:{}", config.server.host, config.server.port);
    info!("Storage: {} (root: {})", config.storage.backend, config.storage.root);
    info!("Qdrant: {} (collection: {})", config.qdrant.url, config.qdrant.collection_name);
    info!("Active Embedding Model: {}", config.ai.active_embedding_model);
    info!("Active Rerank Model: {}", config.ai.active_rerank_model);
    info!("Available Models: {:?}", config.models.keys());

    let storage = storage::Storage::new(&config.storage)
        .context("Failed to initialize storage")?;
    info!("Storage initialized: {} (root: {})", config.storage.backend, config.storage.root);

    // Test: List images in /images directory
    info!("Testing storage: listing /images directory...");
    match storage.list("images/").await {
        Ok(files) => {
            info!("Found {} files in /images", files.len());
            for (i, file) in files.iter().take(5).enumerate() {
                info!("  [{}] {}", i + 1, file);
            }
            if files.len() > 5 {
                info!("  ... and {} more files", files.len() - 5);
            }
        }
        Err(e) => {
            info!("Failed to list /images: {}", e);
        }
    }

    let app = axum::Router::new().route("/", axum::routing::get(|| async { "Hello, MemeforAnyone!" }));
    
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
