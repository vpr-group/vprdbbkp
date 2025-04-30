use anyhow::{anyhow, Result};
use std::{env, fs, path::PathBuf};
use tokio::process::Command;

use super::version::PostgreSQLVersion;

pub struct CommandBuilder {
    pub cache_dir: PathBuf,
    pub version: PostgreSQLVersion,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
}
