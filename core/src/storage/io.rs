use std::{
    io::{Error, ErrorKind, Read, Write},
    sync::mpsc::{channel, Sender},
};

use crate::storage::provider::{StorageProviderCommand, StorageProviderReadResponse};

#[derive(Clone)]
pub struct StorageWriter {
    writer_id: u64,
    command_tx: Sender<StorageProviderCommand>,
    is_closed: bool,
}

impl StorageWriter {
    pub fn new(writer_id: u64, command_tx: Sender<StorageProviderCommand>) -> Self {
        StorageWriter {
            writer_id,
            command_tx,
            is_closed: false,
        }
    }
}

impl Write for StorageWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.is_closed {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Writer has been closed",
            ));
        }

        let (response_tx, response_rx) = channel();

        self.command_tx
            .send(StorageProviderCommand::Write {
                writer_id: self.writer_id,
                data: bytes.to_vec(),
                response: response_tx,
            })
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    format!("Failed to send write command: {}", e),
                )
            })?;

        let result = response_rx.recv().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("Failed to receive write response: {}", e),
            )
        })?;

        result.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Write operation failed: {}", e),
            )
        })?;

        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.is_closed {
            return Ok(());
        }

        let (response_tx, response_rx) = channel();

        self.command_tx
            .send(StorageProviderCommand::CloseWriter {
                writer_id: self.writer_id,
                response: response_tx,
            })
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    format!("Failed to send close command: {}", e),
                )
            })?;

        let result = response_rx.recv().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("Failed to receive close response: {}", e),
            )
        })?;

        result.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Close operation failed: {}", e),
            )
        })?;

        self.is_closed = true;
        Ok(())
    }
}

impl Drop for StorageWriter {
    fn drop(&mut self) {
        if !self.is_closed {
            let (response_tx, _response_rx) = channel();
            let _ = self.command_tx.send(StorageProviderCommand::CloseWriter {
                writer_id: self.writer_id,
                response: response_tx,
            });
            // We don't wait for the response in Drop to avoid blocking
        }
    }
}

#[derive(Clone)]
pub struct StorageReader {
    reader_id: u64,
    command_tx: Sender<StorageProviderCommand>,
    is_closed: bool,
    buffer: Vec<u8>,
}
impl StorageReader {
    pub fn new(reader_id: u64, command_tx: Sender<StorageProviderCommand>) -> Self {
        StorageReader {
            reader_id,
            command_tx,
            is_closed: false,
            buffer: Vec::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn close(&mut self) {
        self.is_closed = true;
        self.buffer.clear();
    }

    fn fetch_more_data(&mut self) -> Result<StorageProviderReadResponse, Error> {
        if self.is_closed {
            return Err(Error::new(ErrorKind::BrokenPipe, "Reader is closed"));
        }

        let (response_tx, response_rx) = channel();

        self.command_tx
            .send(StorageProviderCommand::Read {
                reader_id: self.reader_id,
                response: response_tx,
            })
            .map_err(|e| {
                Error::new(
                    ErrorKind::BrokenPipe,
                    format!("Failed to send read command: {}", e),
                )
            })?;

        let result = response_rx.recv().map_err(|e| {
            Error::new(
                ErrorKind::BrokenPipe,
                format!("Failed to receive read response: {}", e),
            )
        })?;

        result.map_err(|e| Error::new(ErrorKind::Other, format!("Read operation failed: {}", e)))
    }
}

impl Read for StorageReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.is_closed {
            return Err(Error::new(ErrorKind::BrokenPipe, "Reader is closed"));
        }

        if !self.buffer.is_empty() {
            let bytes_to_copy = std::cmp::min(self.buffer.len(), buf.len());
            buf[..bytes_to_copy].copy_from_slice(&self.buffer[..bytes_to_copy]);
            self.buffer.drain(..bytes_to_copy);
            return Ok(bytes_to_copy);
        }

        match self.fetch_more_data() {
            Ok(read_response) => {
                if read_response.is_eof {
                    self.is_closed = true;
                    return Ok(0); // EOF
                }

                if read_response.data.is_empty() {
                    self.is_closed = true;
                    return Ok(0); // EOF
                }

                self.buffer.extend_from_slice(&read_response.data);

                let bytes_to_copy = std::cmp::min(self.buffer.len(), buf.len());
                buf[..bytes_to_copy].copy_from_slice(&self.buffer[..bytes_to_copy]);
                self.buffer.drain(..bytes_to_copy);

                Ok(bytes_to_copy)
            }
            Err(e) => {
                self.is_closed = true;
                Err(e)
            }
        }
    }
}

impl Drop for StorageReader {
    fn drop(&mut self) {
        // No cleanup needed for the reader_id in the provider
        // The provider handles stream cleanup when it reaches EOF
        self.close();
    }
}
