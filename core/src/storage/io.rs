use std::{
    io::{Error, ErrorKind, Read, Write},
    sync::{mpsc::channel, Arc, Mutex as StdMutex},
    thread::{self, JoinHandle},
};

use futures::{StreamExt, TryStreamExt};
use opendal::{BufferStream, Operator, Reader, Writer};
use stream_download_opendal::OpendalStream;
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

pub struct StorageReader {
    worker_handle: Option<JoinHandle<()>>,
}

impl StorageReader {
    pub fn new(operator: Operator, filename: String) -> Self {
        // I didn't find a way to make this simpler. It seems that a reader
        // created from an operator cannot be shared accross threads without
        // producing a deadlock when we need to read from it.

        let worker_handle = thread::spawn(move || {
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

                let bytes = stream.next().await;

                println!("{:?}", bytes);
            });
        });

        StorageReader {
            worker_handle: Some(worker_handle),
        }
    }

    pub async fn read_next_chunk(&self) -> Result<bool, Error> {
        // let mut stream = self.stream.lock().await;
        // let mut buffer = match self.buffer.lock() {
        //     Ok(buffer) => buffer,
        //     Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        // };
        println!("Hehe");

        // match stream.next().await {
        //     Some(result) => match result {
        //         Ok(chunk) => {
        //             buffer.extend_from_slice(&chunk.to_vec());
        //             Ok(true)
        //         }
        //         Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        //     },
        //     None => {
        //         let mut end_of_stream = match self.end_of_stream.lock() {
        //             Ok(end_of_strem) => end_of_strem,
        //             Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        //         };
        //         *end_of_stream = true;
        //         Ok(false)
        //     }
        // }

        // let res = stream.next().await;
        // println!("{:?}", res);

        // let mut end_of_stream = match self.end_of_stream.lock() {
        //     Ok(end_of_strem) => end_of_strem,
        //     Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        // };

        // *end_of_stream = true;
        Ok(false)
    }
}

impl Drop for StorageReader {
    fn drop(&mut self) {
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Read for StorageReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // let this = self.clone();

        // let fut = std::thread::spawn(move || {
        //     let reader = this.reader.clone();
        //     let result: Result<bool, Error> = this.rt.block_on(async {
        //         let reader = this.reader.clone();

        //         let mut stream = reader.into_stream(0..240).await?;

        //         println!("alskdjkl");
        //         let mut buf = [0u8; 10].to_vec();
        //         let res = stream.next().await;
        //         // let res = reader.fetch(vec![0u64..10 as u64]).await;
        //         // println!("{:?}", res);
        //         Ok(true)
        //     });

        //     // let _ = tx.send(result);
        // });

        // let _ = fut.join();

        Ok(0)

        // match rx.recv() {
        //     Ok(result) => match result {
        //         Ok(has_data) => {
        //             if has_data {
        //                 self.read(buf)
        //             } else {
        //                 Ok(0)
        //             }
        //         }
        //         Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        //     },
        //     Err(_) => Err(Error::new(ErrorKind::Other, "Thread communication failed")),
        // }
    }
}
