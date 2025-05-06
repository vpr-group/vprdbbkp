use anyhow::anyhow;
use opendal::{
    layers::LoggingLayer,
    services::{Fs, S3},
    BufferStream, Operator, Writer,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCredentials {
    None,
    Basic {
        username: String,
        password: String,
    },
    AccessKey {
        access_key: String,
        secret_key: String,
    },
    PrivateKey {
        username: String,
        key_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    FileSystem,
    S3,
    // WebDAV,
    // SFTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub id: String,
    pub name: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    pub id: String,
    pub name: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageConfig {
    Local(LocalStorageConfig),
    S3(S3StorageConfig),
}

#[derive(Clone)]
pub struct StorageProvider {
    pub config: StorageConfig,
    pub operator: Arc<Operator>,
}

impl StorageProvider {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        let operator = match &config {
            StorageConfig::Local(config) => {
                let builder = Fs::default().root(&config.location);
                Operator::new(builder)?
                    .layer(LoggingLayer::default())
                    .finish()
            }
            StorageConfig::S3(config) => {
                let mut builder = S3::default()
                    .root(&config.location)
                    .bucket(&config.bucket)
                    .region(&config.region)
                    .access_key_id(&config.access_key)
                    .secret_access_key(&config.secret_key);

                builder = match &config.endpoint {
                    Some(endpoint) => builder.endpoint(endpoint),
                    None => builder,
                };

                Operator::new(builder)?
                    .layer(LoggingLayer::default())
                    .finish()
            }
        };

        Ok(StorageProvider {
            config,
            operator: Arc::new(operator),
        })
    }

    pub async fn test(&self) -> anyhow::Result<bool> {
        match self.operator.list(".").await {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("Storage test failed: {}", e)),
        }
    }

    pub async fn create_writer(&self, filename: &str) -> anyhow::Result<Writer> {
        let writer = self.operator.writer(filename).await?;
        Ok(writer)
    }

    pub async fn create_reader(&self, filename: &str) -> anyhow::Result<BufferStream> {
        let metadata = self.operator.stat(filename).await?;
        let file_size = metadata.content_length() as usize;
        let chunk_size = if file_size > 512 { 512 } else { file_size };

        let stream = self
            .operator
            .reader_with(filename)
            .chunk(chunk_size as usize)
            .await?
            .into_stream(0u64..(file_size as u64))
            .await?;

        Ok(stream)
    }
}

#[cfg(test)]
mod provider_test {
    use super::{LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider};
    use anyhow::Result;
    use dotenv::dotenv;
    use futures::StreamExt;
    use std::{
        env,
        fs::File,
        io::{Cursor, Read},
        path::PathBuf,
    };
    use tempfile::tempdir;

    fn get_local_provider() -> Result<(StorageProvider, PathBuf)> {
        let temp_path = tempdir()?;
        let config = StorageConfig::Local(LocalStorageConfig {
            id: "test".into(),
            name: "local".into(),
            location: temp_path.path().to_str().unwrap().to_string(),
        });
        let provider = StorageProvider::new(config)?;
        Ok((provider, temp_path.into_path()))
    }

    fn get_s3_provider() -> Result<StorageProvider> {
        dotenv().ok();

        let endpoint = env::var("S3_ENDPOINT")
            .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string());

        let config = StorageConfig::S3(S3StorageConfig {
            id: "test".into(),
            name: "s3".into(),
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: Some(endpoint),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            location: "/s3-test".into(),
        });

        let provider = StorageProvider::new(config)?;

        Ok(provider)
    }

    #[tokio::test]
    async fn test_01_local_write() {
        let (provider, path) = get_local_provider().expect("Failed to get local provider");
        let content = "Ceci est un message test".as_bytes();

        let mut reader = Cursor::new(content);
        let mut writer = provider
            .create_writer("test")
            .await
            .expect("Failed to create writer");

        let mut buffer = [0u8; 512];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write(buffer[..n].to_vec())
                        .await
                        .expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.close().await.expect("Failed to close the writer");

        let mut file = File::open(path.join("test")).expect("Unable to open temp file");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Failed to read file");

        assert_eq!(file_content.as_bytes(), content);
    }

    #[tokio::test]
    async fn test_02_s3_write() {
        let provider = get_s3_provider().expect("Failed to get s3 provider");
        let content = "Ceci est un message test".as_bytes();
        let mut reader = Cursor::new(content);

        let mut writer = provider
            .create_writer("test")
            .await
            .expect("Failed to create writer");

        let mut buffer = [0u8; 256];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write(buffer[..n].to_vec())
                        .await
                        .expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.close().await.expect("Failed to close the writer");

        let response = provider
            .operator
            .read("test")
            .await
            .expect("Failed to read response");

        assert_eq!(response.to_bytes(), content);
    }

    #[tokio::test]
    async fn test_03_s3_read() {
        let provider = get_s3_provider().expect("Failed to get s3 provider");
        let content = "Ceci est un message test".as_bytes();
        let mut reader = Cursor::new(content);

        let mut writer = provider
            .create_writer("test")
            .await
            .expect("Failed to create writer");

        let mut buffer = [0u8; 256];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write(buffer[..n].to_vec())
                        .await
                        .expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.close().await.expect("Failed to close the writer");

        let mut bytes = vec![];
        let mut reader = provider
            .create_reader("test")
            .await
            .expect("Failed to create reader");

        while let Some(result) = reader.next().await {
            match result {
                Ok(buffer) => bytes.extend_from_slice(&buffer.to_bytes()),
                Err(_) => break,
            }
        }

        assert_eq!(content, &bytes[0..])
    }
}
