use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use dirs::cache_dir;
use regex::Regex;
use std::{
    borrow::Borrow,
    env,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::{
    compression::CompressionFormat,
    databases::{version::Version, DatabaseConfig},
};

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut prev_is_separator = false;

    for c in input.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_lowercase().next().unwrap());
            prev_is_separator = false;
        } else if !prev_is_separator {
            // Replace any non-alphanumeric character with a hyphen
            slug.push('-');
            prev_is_separator = true;
        }
    }

    // Remove leading and trailing hyphens
    let slug = slug.trim_matches('-');

    slug.to_string()
}

pub fn get_default_backup_name<B>(
    database_config: B,
    compression_format: &CompressionFormat,
) -> String
where
    B: Borrow<DatabaseConfig>,
{
    let borrowed_config: &DatabaseConfig = database_config.borrow();
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d-%H%M%S");
    let uuid_string = Uuid::new_v4().to_string();
    let uuid = uuid_string.split('-').next().unwrap_or("backup");

    let extension = match compression_format {
        CompressionFormat::Zlib => "zip",
        CompressionFormat::Deflate => "zz",
        CompressionFormat::Gzip => "gz",
        CompressionFormat::None => "",
    };

    format!(
        "{}-{}-{}.{}",
        borrowed_config.name, date_str, uuid, extension
    )
}

pub fn get_binaries_base_path(version: &Version) -> PathBuf {
    let db_name = get_db_name(&version);
    let version_name = get_version_name(&version);

    cache_dir()
        .unwrap_or_else(|| env::temp_dir())
        .join("vprdbbkp")
        .join(db_name)
        .join(version_name)
}

pub fn get_db_name(version: &Version) -> String {
    match version {
        Version::PostgreSQL(_) => "postgresql".into(),
        Version::MySql(_) => "mysql".into(),
    }
}

pub fn get_version_name(version: &Version) -> String {
    match version {
        Version::PostgreSQL(version) => version.to_string(),
        Version::MySql(version) => version.to_string(),
    }
}

pub fn extract_timestamp_from_filename(filename: &str) -> Result<DateTime<Utc>> {
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2}-\d{6})-[a-f0-9]+\.(gz|dump|tar|zip|sql)$")
        .map_err(|e| anyhow!("Failed to compile regex: {}", e))?;

    let caps = re.captures(filename).ok_or_else(|| {
        anyhow!(
            "Filename doesn't match expected timestamp format: {}",
            filename
        )
    })?;

    let timestamp_str = caps
        .get(1)
        .ok_or_else(|| anyhow!("Failed to extract timestamp from filename: {}", filename))?
        .as_str();

    let naive_datetime = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d-%H%M%S")
        .map_err(|e| anyhow!("Failed to parse timestamp {}: {}", timestamp_str, e))?;

    let datetime = DateTime::<Utc>::from_utc(naive_datetime, Utc);

    Ok(datetime)
}

pub fn get_arch() -> Result<String> {
    // Get system architecture using std
    let arch = std::env::consts::ARCH;

    // Map architecture to PostgreSQL download architecture
    let result = match arch {
        "x86_64" => "x86_64",
        "amd64" => "x86_64",
        "aarch64" => "arm64",
        "arm64" => "arm64",
        _ => return Err(anyhow!("Unsupported architecture: {}", arch)),
    };

    Ok(result.into())
}

pub fn get_os() -> Result<String> {
    // Get operating system info
    let info = os_info::get();

    // Determine OS type
    let os = match info.os_type() {
        os_info::Type::Linux => "linux",
        os_info::Type::Ubuntu => "linux",
        os_info::Type::Debian => "linux",
        os_info::Type::Fedora => "linux",
        os_info::Type::Redhat => "linux",
        os_info::Type::CentOS => "linux",
        os_info::Type::Alpine => "linux",
        os_info::Type::Mint => "linux",
        os_info::Type::Arch => "linux",
        os_info::Type::Windows => "windows",
        os_info::Type::Macos => "macos",
        _ => {
            return Err(anyhow!(
                "Unsupported operating system: {:?}",
                info.os_type()
            ))
        }
    };

    Ok(os.into())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
