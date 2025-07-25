#[cfg(test)]
mod provider_test {

    use anyhow::Result;
    use dotenv::dotenv;

    use std::{
        env,
        io::{Cursor, Read, Write},
    };
    use tempfile::tempdir;

    use crate::storage::provider::{
        LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider,
    };

    fn get_local_provider() -> Result<StorageProvider> {
        dotenv().ok();

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

        let location = format!("s3_provider_test_{}", chrono::Utc::now().timestamp());

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
            location,
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
