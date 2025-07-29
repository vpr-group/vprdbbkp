use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use log::{debug, error, info, warn};
use opendal::{
    layers::LoggingLayer,
    services::{Fs, S3},
    BufferStream, Metadata, Operator, Writer,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};
use tokio::{runtime::Runtime, sync::oneshot};

use crate::{common::extract_timestamp_from_filename, storage::Entry};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOptions {
    pub latest_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct StorageProviderReadResponse {
    pub data: Vec<u8>,
    pub size: usize,
    pub is_eof: bool,
}

#[derive(Debug)]
pub enum StorageProviderCommand {
    List {
        path: String,
        options: ListOptions,
        response: oneshot::Sender<Result<Vec<Entry>>>,
    },
    CreateWriter {
        path: String,
        concurrency: usize,
        response: oneshot::Sender<Result<u64>>,
    },
    Write {
        writer_id: u64,
        data: Vec<u8>,
        response: Sender<Result<()>>,
    },
    CloseWriter {
        writer_id: u64,
        response: Sender<Result<Metadata>>,
    },
    CreateReader {
        path: String,
        response: oneshot::Sender<Result<u64>>,
    },
    Read {
        reader_id: u64,
        response: Sender<Result<StorageProviderReadResponse>>,
    },
    Delete {
        path: String,
        response: oneshot::Sender<Result<()>>,
    },
    Test {
        response: oneshot::Sender<Result<bool>>,
    },
    Cleanup {
        retention_days: u64,
        dry_run: bool,
        response: oneshot::Sender<Result<(usize, u64)>>,
    },
    Shutdown {
        response: oneshot::Sender<Result<()>>,
    },
}

#[derive(Clone)]
pub struct StorageProvider {
    command_tx: Sender<StorageProviderCommand>,
    _worker_handle: Arc<Option<JoinHandle<Result<()>>>>,
}

impl StorageProvider {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        let (command_tx, command_rx) = channel::<StorageProviderCommand>();
        let config_clone = config.clone();

        let worker_handle = thread::spawn(move || -> Result<()> {
            let rt = Runtime::new()?;
            rt.block_on(async {
                let operator = match &config_clone {
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

                let mut writers: HashMap<u64, Writer> = HashMap::new();
                let mut next_writer_id = 1u64;

                let mut streams: HashMap<u64, BufferStream> = HashMap::new();
                let mut next_stream_id = 1u64;

                while let Ok(command) = command_rx.recv() {
                    match command {
                        StorageProviderCommand::List {
                            path,
                            options,
                            response,
                        } => {
                            debug!("Processing List command for path: {}", path);

                            let limit = options.limit.unwrap_or(1000);
                            let latest_only = options.latest_only.unwrap_or(false);

                            let result =
                                operator.list_with(&path).recursive(true).limit(limit).await;

                            let _ = response.send(match result {
                                Ok(entries) => {
                                    let mut filtered_results: Vec<Entry> = entries
                                        .into_iter()
                                        .map(|opendal_entry| {
                                            let mut entry = Entry::from(&opendal_entry);
                                            // Get content length for local files
                                            if let StorageConfig::Local(local_config) =
                                                &config_clone
                                            {
                                                let full_path = Path::new(&local_config.location)
                                                    .join(&entry.path);
                                                if let Ok(metadata) = fs::metadata(&full_path) {
                                                    entry.metadata.content_length = metadata.len();
                                                }
                                            }
                                            entry
                                        })
                                        .filter(|entry| entry.metadata.is_file)
                                        .collect();

                                    // Sort by timestamp (newest first)
                                    filtered_results.sort_by(|a, b| {
                                        let a_timestamp =
                                            extract_timestamp_from_filename(&a.metadata.name)
                                                .unwrap_or_else(|_| {
                                                    DateTime::<Utc>::from(SystemTime::UNIX_EPOCH)
                                                });
                                        let b_timestamp =
                                            extract_timestamp_from_filename(&b.metadata.name)
                                                .unwrap_or_else(|_| {
                                                    DateTime::<Utc>::from(SystemTime::UNIX_EPOCH)
                                                });
                                        b_timestamp.cmp(&a_timestamp)
                                    });

                                    if latest_only {
                                        match filtered_results.first() {
                                            Some(entry) => Ok(vec![entry.clone()]),
                                            None => Err(anyhow!("No entry found")),
                                        }
                                    } else {
                                        Ok(filtered_results)
                                    }
                                }
                                Err(error) => Err(anyhow!("{}", error)),
                            });
                        }

                        StorageProviderCommand::CreateWriter {
                            path,
                            concurrency,
                            response,
                        } => {
                            debug!("Processing CreateWriter command for path: {}", path);
                            match operator.writer_with(&path).concurrent(concurrency).await {
                                Ok(writer) => {
                                    let writer_id = next_writer_id;
                                    next_writer_id += 1;
                                    writers.insert(writer_id, writer);
                                    let _ = response.send(Ok(writer_id));
                                }
                                Err(e) => {
                                    let _ = response.send(Err(anyhow!("{}", e)));
                                }
                            }
                        }

                        StorageProviderCommand::Write {
                            writer_id,
                            data,
                            response,
                        } => {
                            debug!(
                                "Processing Write command for writer {}: {} bytes",
                                writer_id,
                                data.len()
                            );
                            if let Some(writer) = writers.get_mut(&writer_id) {
                                let result = writer.write(data).await;
                                let _ = response.send(result.map_err(|e| anyhow!("{}", e)));
                            } else {
                                let _ =
                                    response.send(Err(anyhow!("Writer {} not found", writer_id)));
                            }
                        }

                        StorageProviderCommand::CloseWriter {
                            writer_id,
                            response,
                        } => {
                            debug!("Processing CloseWriter command for writer {}", writer_id);
                            if let Some(mut writer) = writers.remove(&writer_id) {
                                let result = writer.close().await;
                                let _ = response.send(result.map_err(|e| anyhow!("{}", e)));
                            } else {
                                let _ =
                                    response.send(Err(anyhow!("Writer {} not found", writer_id)));
                            }
                        }

                        StorageProviderCommand::CreateReader { path, response } => {
                            debug!("Processing CreateReader command for path: {}", path);

                            match operator.stat(&path).await {
                                Ok(metadata) => {
                                    let file_size = metadata.content_length() as usize;
                                    let chunk_size = if file_size > 512 { 512 } else { file_size };

                                    match operator
                                        .reader_with(&path)
                                        .chunk(chunk_size)
                                        .concurrent(2)
                                        .await
                                    {
                                        Ok(reader) => {
                                            match reader.into_stream(0u64..(file_size as u64)).await
                                            {
                                                Ok(stream) => {
                                                    let reader_id = next_stream_id;
                                                    next_stream_id += 1;
                                                    streams.insert(reader_id, stream);
                                                    let _ = response.send(Ok(reader_id));
                                                }
                                                Err(e) => {
                                                    let _ = response.send(Err(anyhow!("{}", e)));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = response.send(Err(anyhow!("{}", e)));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = response.send(Err(anyhow!("{}", e)));
                                }
                            }
                        }

                        StorageProviderCommand::Read {
                            reader_id,
                            response,
                        } => {
                            debug!("Processing Read command for reader: {}", reader_id);
                            if let Some(stream) = streams.get_mut(&reader_id) {
                                let result = match stream.next().await {
                                    Some(Ok(chunk)) => {
                                        let data = chunk.to_bytes().to_vec();
                                        let size = data.len();
                                        Ok(StorageProviderReadResponse {
                                            data,
                                            size,
                                            is_eof: false,
                                        })
                                    }
                                    Some(Err(e)) => Err(anyhow!("{}", e)),
                                    None => {
                                        // End of stream, remove it
                                        streams.remove(&reader_id);
                                        Ok(StorageProviderReadResponse {
                                            data: Vec::new(),
                                            size: 0,
                                            is_eof: true,
                                        })
                                    }
                                };

                                let _ = response.send(result);
                            } else {
                                let _ =
                                    response.send(Err(anyhow!("Reader {} not found", reader_id)));
                            }
                        }

                        StorageProviderCommand::Delete { path, response } => {
                            debug!("Processing Delete command for path: {}", path);
                            let result = operator.delete(&path).await;
                            let _ = response.send(result.map_err(|e| anyhow!("{}", e)));
                        }

                        StorageProviderCommand::Test { response } => {
                            debug!("Processing Test command");
                            let result = operator.list_with("/").recursive(true).limit(1).await;
                            let _ = response.send(match result {
                                Ok(_) => Ok(true),
                                Err(e) => Err(anyhow!("{}", e)),
                            });
                        }

                        StorageProviderCommand::Cleanup {
                            retention_days,
                            dry_run,
                            response,
                        } => {
                            debug!("Processing Cleanup command");

                            // Get all files
                            let list_result =
                                operator.list_with("").recursive(true).limit(10000).await;

                            let result = match list_result {
                                Ok(entries) => {
                                    let cutoff = SystemTime::now()
                                        .checked_sub(Duration::from_secs(retention_days * 86400))
                                        .ok_or_else(|| {
                                            anyhow!("Failed to calculate cutoff date")
                                        })?;

                                    let cutoff_datetime: DateTime<Utc> = cutoff.into();

                                    let mut deleted_count = 0;
                                    let mut deleted_size = 0;

                                    for opendal_entry in entries {
                                        let entry = Entry::from(&opendal_entry);
                                        if !entry.metadata.is_file {
                                            continue;
                                        }

                                        match extract_timestamp_from_filename(&entry.metadata.name)
                                        {
                                            Ok(timestamp) => {
                                                if timestamp < cutoff_datetime {
                                                    let size = entry.metadata.content_length;
                                                    deleted_size += size;
                                                    deleted_count += 1;

                                                    if !dry_run {
                                                        if let Err(e) =
                                                            operator.delete(&entry.path).await
                                                        {
                                                            error!(
                                                                "Failed to delete {}: {}",
                                                                entry.path, e
                                                            );
                                                        } else {
                                                            info!(
                                                                "Successfully deleted {}",
                                                                entry.path
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                warn!(
                                                    "Failed to extract timestamp from {}",
                                                    entry.metadata.name
                                                );
                                            }
                                        }
                                    }

                                    Ok((deleted_count, deleted_size))
                                }
                                Err(e) => Err(anyhow!("{}", e)),
                            };

                            let _ = response.send(result);
                        }

                        StorageProviderCommand::Shutdown { response } => {
                            debug!("Processing Shutdown command");

                            // Close all remaining writers
                            for (writer_id, mut writer) in writers.drain() {
                                debug!("Closing remaining writer {}", writer_id);
                                if let Err(e) = writer.close().await {
                                    error!("Error closing writer {}: {}", writer_id, e);
                                }
                            }

                            // Clear all streams
                            streams.clear();

                            let _ = response.send(Ok(()));
                            break; // Exit the command loop
                        }
                    }
                }

                debug!("Provider worker thread exiting");
                Ok(())
            })
        });

        Ok(StorageProvider {
            command_tx,
            _worker_handle: Arc::new(Some(worker_handle)),
        })
    }

    pub async fn test(&self) -> Result<bool> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::Test {
            response: response_tx,
        })?;

        let _ = response_rx.await?;
        Ok(true)
    }

    pub async fn list(&self) -> Result<Vec<Entry>> {
        self.list_with_options(ListOptions {
            latest_only: None,
            limit: None,
        })
        .await
    }

    pub async fn list_with_options(&self, options: ListOptions) -> Result<Vec<Entry>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::List {
            path: String::new(),
            options,
            response: response_tx,
        })?;

        response_rx.await?
    }

    pub async fn create_writer(&self, path: &str) -> Result<StorageWriter> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::CreateWriter {
            path: path.to_string(),
            response: response_tx,
            concurrency: 5,
        })?;

        let writer_id = response_rx.await??;
        Ok(StorageWriter::new(writer_id, self.command_tx.clone()))
    }

    pub async fn create_reader(&self, filename: &str) -> Result<StorageReader> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::CreateReader {
            path: filename.to_string(),
            response: response_tx,
        })?;

        let reader_id = response_rx.await??;
        Ok(StorageReader::new(reader_id, self.command_tx.clone()))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::Delete {
            path: path.to_string(),
            response: response_tx,
        })?;

        response_rx.await?
    }

    pub async fn cleanup(&self, retention_days: u64, dry_run: bool) -> Result<(usize, u64)> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::Cleanup {
            retention_days,
            dry_run,
            response: response_tx,
        })?;

        response_rx.await?
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx.send(StorageProviderCommand::Shutdown {
            response: response_tx,
        })?;

        response_rx.await?
    }
}

impl Drop for StorageProvider {
    fn drop(&mut self) {
        // Attempt graceful shutdown
        let _ = self.shutdown();
    }
}
