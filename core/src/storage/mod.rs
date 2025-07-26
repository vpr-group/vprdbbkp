use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod io;
pub mod provider;
mod test;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EntryMode {
    FILE,
    DIR,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub mode: EntryMode,
    pub name: String,
    pub is_file: bool,
    pub is_current: Option<bool>,
    pub is_deleted: bool,
    pub cache_control: Option<String>,
    pub content_disposition: Option<String>,
    pub content_length: u64,
    pub content_md5: Option<String>,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub metadata: EntryMetadata,
}

impl From<&opendal::Entry> for Entry {
    fn from(opendal_entry: &opendal::Entry) -> Self {
        let path = opendal_entry.path().to_string();
        let metadata = opendal_entry.metadata();

        Self {
            path,
            metadata: EntryMetadata {
                name: opendal_entry.name().to_string(),
                cache_control: metadata.cache_control().map(|it| it.to_string()),
                content_disposition: metadata.content_disposition().map(|it| it.to_string()),
                content_encoding: metadata.content_encoding().map(|it| it.to_string()),
                content_length: metadata.content_length(),
                content_md5: metadata.content_md5().map(|it| it.to_string()),
                content_type: metadata.content_type().map(|it| it.to_string()),
                etag: metadata.etag().map(|it| it.to_string()),
                is_current: metadata.is_current(),
                is_deleted: metadata.is_deleted(),
                is_file: metadata.is_file(),
                last_modified: metadata.last_modified(),
                mode: match metadata.mode() {
                    opendal::EntryMode::DIR => EntryMode::DIR,
                    opendal::EntryMode::FILE => EntryMode::FILE,
                    opendal::EntryMode::Unknown => EntryMode::Unknown,
                },
                version: metadata.version().map(|it| it.to_string()),
            },
        }
    }
}
