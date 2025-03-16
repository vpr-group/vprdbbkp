use opendal::{Entry, EntryMode, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEntry {
    path: String,
    metadata: SerializableMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMetadata {
    mode: String,
    is_current: Option<bool>,
    is_deleted: bool,
    cache_control: Option<String>,
    content_disposition: Option<String>,
    content_length: u64,
    content_md5: Option<String>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    etag: Option<String>,
    last_modified: Option<String>,
    version: Option<String>,
    user_metadata: Option<HashMap<String, String>>,
}

impl From<Entry> for SerializableEntry {
    fn from(entry: Entry) -> Self {
        let path = entry.path().to_string();
        let metadata = SerializableMetadata::from(entry.metadata());

        Self { path, metadata }
    }
}

impl From<&Entry> for SerializableEntry {
    fn from(entry: &Entry) -> Self {
        let path = entry.path().to_string();
        let metadata = SerializableMetadata::from(entry.metadata());

        Self { path, metadata }
    }
}

impl From<&Metadata> for SerializableMetadata {
    fn from(metadata: &Metadata) -> Self {
        // Convert EntryMode to string
        let mode = match metadata.mode() {
            EntryMode::FILE => "FILE".to_string(),
            EntryMode::DIR => "DIR".to_string(),
            _ => "UNKNOWN".to_string(),
        };

        // Format last_modified as ISO 8601 string if available
        let last_modified = metadata.last_modified().map(|dt| dt.to_rfc3339());

        Self {
            mode,
            is_current: metadata.is_current(),
            is_deleted: metadata.is_deleted(),
            cache_control: metadata.cache_control().map(String::from),
            content_disposition: metadata.content_disposition().map(String::from),
            content_length: metadata.content_length(),
            content_md5: metadata.content_md5().map(String::from),
            content_type: metadata.content_type().map(String::from),
            content_encoding: metadata.content_encoding().map(String::from),
            etag: metadata.etag().map(String::from),
            last_modified,
            version: metadata.version().map(String::from),
            user_metadata: metadata.user_metadata().cloned(),
        }
    }
}

impl SerializableEntry {
    /// Path of entry. Path is relative to operator's root.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Fetch metadata of this entry.
    pub fn metadata(&self) -> &SerializableMetadata {
        &self.metadata
    }
}

impl SerializableMetadata {
    /// Returns whether the entry is a file
    pub fn is_file(&self) -> bool {
        self.mode == "FILE"
    }

    /// Returns whether the entry is a directory
    pub fn is_dir(&self) -> bool {
        self.mode == "DIR"
    }

    /// Content length of this entry
    pub fn content_length(&self) -> u64 {
        self.content_length
    }

    /// Content type of this entry
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Last modified time as RFC 3339 string
    pub fn last_modified(&self) -> Option<&str> {
        self.last_modified.as_deref()
    }

    /// ETag of this entry
    pub fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }
}

/// Helper function to convert Vec<Entry> to Vec<SerializableEntry>
pub fn entries_to_serializable(entries: Vec<Entry>) -> Vec<SerializableEntry> {
    entries.into_iter().map(SerializableEntry::from).collect()
}

/// Helper function to convert Vec<&Entry> to Vec<SerializableEntry>
pub fn entry_refs_to_serializable(entries: &[Entry]) -> Vec<SerializableEntry> {
    entries.iter().map(SerializableEntry::from).collect()
}
