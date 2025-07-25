use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::{self, Seek, SeekFrom, Write};

#[derive(Clone, Serialize, Deserialize)]
pub enum CompressionFormat {
    Gzip,
    Zlib,
    Deflate,
    None,
}

pub enum Compressor<W: Write + Send + Unpin> {
    Gzip(GzEncoder<W>),
    Zlib(ZlibEncoder<W>),
    Deflate(DeflateEncoder<W>),
    None(W),
}

impl<W: Write + Send + Unpin> Compressor<W> {
    pub fn new(writer: W, format: CompressionFormat, level: Compression) -> Self {
        match format {
            CompressionFormat::Gzip => Compressor::Gzip(GzEncoder::new(writer, level)),
            CompressionFormat::Zlib => Compressor::Zlib(ZlibEncoder::new(writer, level)),
            CompressionFormat::Deflate => Compressor::Deflate(DeflateEncoder::new(writer, level)),
            CompressionFormat::None => Compressor::None(writer),
        }
    }

    pub fn finish(self) -> io::Result<W> {
        match self {
            Compressor::Gzip(encoder) => encoder.finish(),
            Compressor::Zlib(encoder) => encoder.finish(),
            Compressor::Deflate(encoder) => encoder.finish(),
            Compressor::None(writer) => Ok(writer),
        }
    }
}

impl<W: Write + Send + Unpin> Write for Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Compressor::Gzip(ref mut encoder) => encoder.write(buf),
            Compressor::Zlib(ref mut encoder) => encoder.write(buf),
            Compressor::Deflate(ref mut encoder) => encoder.write(buf),
            Compressor::None(ref mut writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Compressor::Gzip(ref mut encoder) => encoder.flush(),
            Compressor::Zlib(ref mut encoder) => encoder.flush(),
            Compressor::Deflate(ref mut encoder) => encoder.flush(),
            Compressor::None(ref mut writer) => writer.flush(),
        }
    }
}

use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use std::io::Read;

pub enum Decompressor<R: Read + Send + Unpin> {
    Gzip(GzDecoder<R>),
    Zlib(ZlibDecoder<R>),
    Deflate(DeflateDecoder<R>),
    None(R),
}

impl<R: Read + Send + Unpin> Decompressor<R> {
    pub fn new(reader: R, format: CompressionFormat) -> Self {
        match format {
            CompressionFormat::Gzip => Decompressor::Gzip(GzDecoder::new(reader)),
            CompressionFormat::Zlib => Decompressor::Zlib(ZlibDecoder::new(reader)),
            CompressionFormat::Deflate => Decompressor::Deflate(DeflateDecoder::new(reader)),
            CompressionFormat::None => Decompressor::None(reader),
        }
    }

    pub fn detect_format(mut reader: R) -> io::Result<(CompressionFormat, R)>
    where
        R: Read + Seek,
    {
        let mut signature = [0u8; 3];
        let start_pos = reader.stream_position()?;
        let bytes_read = reader.read(&mut signature)?;
        reader.seek(SeekFrom::Start(start_pos))?;

        if bytes_read < 2 {
            return Ok((CompressionFormat::None, reader));
        }

        if signature[0] == 0x1F && signature[1] == 0x8B {
            Ok((CompressionFormat::Gzip, reader))
        } else if signature[0] == 0x78
            && (signature[1] == 0x01 || signature[1] == 0x9C || signature[1] == 0xDA)
        {
            Ok((CompressionFormat::Zlib, reader))
        } else {
            Ok((CompressionFormat::None, reader))
        }
    }

    pub fn into_inner(self) -> R {
        match self {
            Decompressor::Gzip(decoder) => decoder.into_inner(),
            Decompressor::Zlib(decoder) => decoder.into_inner(),
            Decompressor::Deflate(decoder) => decoder.into_inner(),
            Decompressor::None(reader) => reader,
        }
    }
}

impl<R: Read + Send + Unpin> Read for Decompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Decompressor::Gzip(ref mut decoder) => decoder.read(buf),
            Decompressor::Zlib(ref mut decoder) => decoder.read(buf),
            Decompressor::Deflate(ref mut decoder) => decoder.read(buf),
            Decompressor::None(ref mut reader) => reader.read(buf),
        }
    }
}

#[cfg(test)]
mod compression_test {
    use std::io::{Cursor, Read, Write};

    use flate2::Compression;

    use crate::compression::{CompressionFormat, Decompressor};

    use super::Compressor;

    #[test]
    fn compress() {
        let message = "Ceci est un texte test";
        let bytes = vec![];
        let mut compressor = Compressor::new(bytes, CompressionFormat::Zlib, Compression::best());

        compressor
            .write_all(message.as_bytes())
            .expect("Failed to write bytes");

        let mut res = compressor.finish().expect("Unable to finish compressor");
        res.flush().expect("Failed to flush");

        let reader = Cursor::new(res);
        let mut decompressor = Decompressor::new(reader, CompressionFormat::Zlib);

        let mut buf = [0u8; 512];
        let n = decompressor.read(&mut buf).expect("Failed to read bytes");
        let decompressed_bytes = &buf[..n];

        assert_eq!(message.as_bytes(), decompressed_bytes);
    }
}
