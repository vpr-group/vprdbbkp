use std::{
    io::{Error, ErrorKind, Read, Write},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex as StdMutex,
    },
    thread::{self, JoinHandle},
};

use futures::StreamExt;
use opendal::{Operator, Writer};
use tokio::sync::Mutex as TokioMutex;

#[derive(Clone)]
pub struct StorageWriter {
    writer: Arc<TokioMutex<Writer>>,
}

impl StorageWriter {
    pub fn new(writer: Writer) -> Self {
        StorageWriter {
            writer: Arc::new(TokioMutex::new(writer)),
        }
    }

    async fn write_async(&mut self, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().await;
        let data_to_write = buf.to_owned();
        writer.write(data_to_write).await?;

        Ok(buf.len())
    }

    async fn flush_async(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().await;
        writer.close().await?;

        Ok(())
    }
}

impl Write for StorageWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let buf_copy = buf.to_vec();
        let mut this = self.clone();
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result: Result<usize, Error> = rt.block_on(async {
                let len = this
                    .write_async(&buf_copy.clone())
                    .await
                    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                Ok(len)
            });

            let _ = tx.send(result);
        });

        match rx.recv() {
            Ok(result) => match result {
                Ok(size) => Result::Ok(size),
                Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "Thread communication failed")),
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        let mut this = self.clone();
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result: Result<(), Error> = rt.block_on(async {
                let result = this
                    .flush_async()
                    .await
                    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                Ok(result)
            });

            let _ = tx.send(result);
        });

        match rx.recv() {
            Ok(result) => match result {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
            },
            Err(_) => Err(Error::new(ErrorKind::Other, "Thread communication failed")),
        }
    }
}

#[derive(Debug)]
enum FetchResult {
    DataAvailable,
    EndOfStream,
    Error(String),
}

enum ReadRequest {
    FetchMoreData(Sender<FetchResult>),
    Stop,
}

pub struct StorageReader {
    worker_handle: Option<JoinHandle<()>>,
    buffer: Arc<StdMutex<Vec<u8>>>,
    tx: Sender<ReadRequest>,
}

impl StorageReader {
    pub fn new(operator: Operator, filename: String) -> Self {
        // I didn't find a way to make this simpler. It seems that a reader
        // created from an operator cannot be shared accross threads without
        // producing a deadlock when we need to read from it.

        let buffer = Arc::new(StdMutex::new(Vec::new()));
        let (tx, rx) = channel();

        let worker_handle = thread::spawn({
            let buffer = buffer.clone();

            move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        eprintln!("Failed to create runtime: {}", e);
                        return;
                    }
                };

                rt.block_on(async {
                    let metadata = operator.stat(filename.as_str()).await.unwrap();
                    let file_size = metadata.content_length() as usize;
                    let chunk_size = if file_size > 512 { 512 } else { file_size };

                    let mut stream = operator
                        .reader_with(filename.as_str())
                        .chunk(chunk_size as usize)
                        .await
                        .unwrap()
                        .into_stream(0u64..(file_size as u64))
                        .await
                        .unwrap();

                    while let Ok(request) = rx.recv() {
                        match request {
                            ReadRequest::FetchMoreData(tx) => {
                                let result = match stream.next().await {
                                    Some(Ok(chunk)) => {
                                        if let Ok(mut buffer) = buffer.lock() {
                                            buffer.extend_from_slice(&chunk.to_bytes());
                                            FetchResult::DataAvailable
                                        } else {
                                            FetchResult::Error(format!("Failed to lock buffer"))
                                        }
                                    }
                                    Some(Err(e)) => {
                                        FetchResult::Error(format!("Error reading chunk: {}", e))
                                    }
                                    None => FetchResult::EndOfStream,
                                };

                                let _ = tx.send(result);
                            }
                            ReadRequest::Stop => {
                                break;
                            }
                        }
                    }
                });
            }
        });

        StorageReader {
            worker_handle: Some(worker_handle),
            buffer,
            tx,
        }
    }
}

impl Drop for StorageReader {
    fn drop(&mut self) {
        let _ = self.tx.send(ReadRequest::Stop);

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Read for StorageReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        {
            let mut buffer = match self.buffer.lock() {
                Ok(buffer) => buffer,
                Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to lock buffer")),
            };

            if !buffer.is_empty() {
                let bytes_to_copy = std::cmp::min(buffer.len(), buf.len());
                buf[..bytes_to_copy].copy_from_slice(&buffer[..bytes_to_copy]);
                buffer.drain(..bytes_to_copy);

                return Ok(bytes_to_copy);
            }
        }

        let (response_tx, response_rx) = channel();

        match self.tx.send(ReadRequest::FetchMoreData(response_tx)) {
            Ok(()) => {}
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        let fetch_result = match response_rx.recv() {
            Ok(response) => response,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        match fetch_result {
            FetchResult::DataAvailable => self.read(buf),
            FetchResult::EndOfStream => Ok(0),
            FetchResult::Error(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    }
}
