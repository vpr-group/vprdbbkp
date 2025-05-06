use anyhow::anyhow;
use futures::executor::block_on;
use opendal::{layers::LoggingLayer, services::Fs, Operator};
use serde::{Deserialize, Serialize};
use std::{io::Write, pin::Pin, sync::Arc};

pub struct OpenDALStreamingWriter {
    writer: Pin<Box<opendal::Writer>>,
}

impl OpenDALStreamingWriter {
    pub fn new(writer: opendal::Writer) -> Self {
        OpenDALStreamingWriter {
            writer: Pin::new(Box::new(writer)),
        }
    }
}

impl Write for OpenDALStreamingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        block_on({
            async {
                let buf = buf.to_vec();
                self.writer
                    .write(buf.clone())
                    .await
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                Ok(buf.len())
            }
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = block_on(async {
            self.writer
                .close()
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        });

        Ok(())
    }
}

impl Drop for OpenDALStreamingWriter {
    fn drop(&mut self) {
        let _ = block_on(async {
            self.writer
                .close()
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    FileSystem,
    // S3,
    // WebDAV,
    // SFTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub id: String,
    pub name: String,
    pub storage_type: StorageType,
    pub location: String,
}

#[derive(Clone)]
pub struct StorageProvider {
    pub config: StorageConfig,
    operator: Arc<Operator>,
}

impl StorageProvider {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        let operator = match config.storage_type {
            StorageType::FileSystem => {
                let builder = Fs::default().root(&config.location);

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

    pub async fn create_writer(&self, filename: &str) -> anyhow::Result<Box<dyn Write>> {
        let backup_dir = "backups";
        let path = format!("{}/{}", backup_dir, filename);
        let operator_writer = self.operator.writer(&path).await?;
        let writer = OpenDALStreamingWriter::new(operator_writer);

        Ok(Box::new(writer))
    }
}

#[cfg(test)]
mod provider_test {
    use super::{StorageConfig, StorageProvider, StorageType};
    use anyhow::Result;
    use std::io::{Cursor, Read};

    fn get_local_provider() -> Result<StorageProvider> {
        let config = StorageConfig {
            id: "test".into(),
            name: "local".into(),
            storage_type: StorageType::FileSystem,
            location: "./".into(),
        };
        let provider = StorageProvider::new(config)?;
        Ok(provider)
    }

    #[tokio::test]
    async fn test_01_write() {
        let provider = get_local_provider().expect("Failed to get local provider");
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
                        .write_all(&buffer[..n])
                        .expect("Failed to write bytes");
                }
                Err(e) => (),
            }
        }

        println!("Write");
    }
}
