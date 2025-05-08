use anyhow::Result;
use opendal::{
    layers::LoggingLayer,
    services::{Fs, S3},
    BufferStream, Operator,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

use super::io::{StorageReader, StorageWriter};

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
    pub operator: Operator,
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

        Ok(StorageProvider { config, operator })
    }

    pub async fn test(&self) -> Result<bool> {
        self.operator
            .list_with("/")
            .recursive(true)
            .limit(1)
            .await?;

        Ok(true)
    }

    pub async fn create_writer(&self, filename: &str) -> Result<Box<dyn Write>> {
        let op_writer = self.operator.writer(filename).await?;
        Ok(Box::new(StorageWriter::new(op_writer)))
    }

    pub async fn create_stream(&self, filename: &str) -> Result<Arc<Mutex<BufferStream>>> {
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

        Ok(Arc::new(Mutex::new(stream)))
    }

    pub async fn create_reader(&self, filename: &str) -> Result<Box<dyn Read>> {
        Ok(Box::new(StorageReader::new(
            self.operator.clone(),
            filename.to_string(),
        )))
    }
}

#[cfg(test)]
mod provider_test {
    use super::{LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider};
    use anyhow::Result;
    use dotenv::dotenv;

    use std::{
        env,
        io::{Cursor, Read, Write},
    };
    use tempfile::tempdir;

    fn get_local_provider() -> Result<StorageProvider> {
        let temp_path = tempdir()?;
        let config = StorageConfig::Local(LocalStorageConfig {
            id: "test".into(),
            name: "local".into(),
            location: temp_path.path().to_str().unwrap().to_string(),
        });
        let provider = StorageProvider::new(config)?;
        Ok(provider)
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
    async fn test_01_local() {
        let provider = get_local_provider().expect("Failed to get local provider");
        let content = "Ceci est un message test".as_bytes();

        let mut content_reader = Cursor::new(content);
        let mut writer = provider
            .create_writer("test")
            .await
            .expect("Failed to create writer");

        let mut buffer = [0u8; 10];

        loop {
            match content_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write(&buffer[..n].to_vec())
                        .expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.flush().expect("Failed to flush the writer");

        let is_connected = provider.test().await.expect("Failed to test the provider");
        assert!(is_connected);

        let mut reader = provider
            .create_reader("test")
            .await
            .expect("Failed to create reader");

        let mut reader_content = vec![];

        while let Ok(n) = reader.read(&mut buffer) {
            if (n) > 0 {
                reader_content.extend_from_slice(&buffer[..n]);
            } else {
                break;
            }
        }

        assert_eq!(reader_content.as_slice(), content);
    }

    #[tokio::test]
    async fn test_02_s3() {
        let provider = get_s3_provider().expect("Failed to get s3 provider");

        let mut writer = provider
            .create_writer("test-01")
            .await
            .expect("Failed to create writer");

        let content = "Ceci est un message test".as_bytes();
        let mut reader = Cursor::new(content);
        let mut buffer = [0u8; 10];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer.write(&buffer[..n]).expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.flush().expect("Failed to flush");

        let mut reader = provider
            .create_reader("test-01")
            .await
            .expect("Failed to create reader");

        let mut reader_content = vec![];

        while let Ok(n) = reader.read(&mut buffer) {
            if (n) > 0 {
                reader_content.extend_from_slice(&buffer[..n]);
            } else {
                break;
            }
        }

        assert_eq!(content, reader_content.as_slice());
    }

    #[tokio::test]
    async fn test_03_list() {
        let provider = get_s3_provider().expect("Failed to get s3 provider");

        let entries = provider
            .operator
            .list_with("/")
            .recursive(true)
            .limit(10)
            .await
            .expect("Failed to list dumps in");

        println!("{:?}", entries);
    }
}
