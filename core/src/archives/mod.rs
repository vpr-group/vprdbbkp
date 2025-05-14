use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod installer;
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseArchives {
    metadata: Metadata,
    databases: Vec<Database>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    schema_version: String,
    last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Database {
    database: String,
    archives: Vec<Archive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archive {
    version: Version,
    platforms: HashMap<String, Platform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Version {
    major: u32,
    minor: Option<u32>,
    patch: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Platform {
    url: String,
}
