use anyhow::{Context, Result};
use futures::StreamExt;
use opendal::{Operator, services};
use crate::config::StorageConfig;

#[derive(Clone)]
pub struct Storage {
    operator: Operator,
}

impl Storage {
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let operator = match config.backend.as_str() {
            "fs" => {
                let builder = services::Fs::default()
                    .root(&config.root);
                Operator::new(builder)?
                    .finish()
            }
            "s3" => {
                let mut builder = services::S3::default()
                    .bucket(&config.root);
                
                if let Some(endpoint) = &config.s3_endpoint {
                    builder = builder.endpoint(endpoint);
                }
                if let Some(region) = &config.s3_region {
                    builder = builder.region(region);
                }
                if let Some(access_key) = &config.s3_access_key {
                    builder = builder.access_key_id(access_key);
                }
                if let Some(secret_key) = &config.s3_secret_key {
                    builder = builder.secret_access_key(secret_key);
                }
                
                Operator::new(builder)?
                    .finish()
            }
            _ => anyhow::bail!("Unsupported storage backend: {}", config.backend),
        };

        Ok(Self { operator })
    }

    pub async fn list(&self, path: &str) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        let mut lister = self.operator
            .lister(path)
            .await
            .context("Failed to create lister")?;

        while let Some(entry) = lister.next().await {
            let entry = entry?;
            entries.push(entry.path().to_string());
        }

        Ok(entries)
    }

    pub async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let buffer = self.operator
            .read(path)
            .await
            .context(format!("Failed to read file: {}", path))?;
        Ok(buffer.to_vec())
    }

    pub async fn write(&self, path: &str, data: Vec<u8>) -> Result<()> {
        self.operator
            .write(path, data)
            .await
            .context(format!("Failed to write file: {}", path))?;
        Ok(())
    }

    pub async fn exists(&self, path: &str) -> Result<bool> {
        match self.operator.stat(path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        self.operator
            .delete(path)
            .await
            .context(format!("Failed to delete file: {}", path))
    }

    pub async fn stat(&self, path: &str) -> Result<opendal::Metadata> {
        self.operator
            .stat(path)
            .await
            .context(format!("Failed to stat file: {}", path))
    }
}
