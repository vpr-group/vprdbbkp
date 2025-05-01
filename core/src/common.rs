use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::{borrow::Borrow, path::Path};
use uuid::Uuid;

use crate::databases::configs::SourceConfig;

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

pub fn get_source_name<B>(source_config: B) -> String
where
    B: Borrow<SourceConfig>,
{
    match source_config.borrow() {
        SourceConfig::PG(config) => {
            format!("{}-{}", slugify(&config.name), slugify(&config.database),)
        }
        SourceConfig::MariaDB(config) => {
            format!("{}-{}", slugify(&config.name), slugify(&config.database),)
        }
    }
}

pub fn get_filename<B>(source_config: B) -> String
where
    B: Borrow<SourceConfig>,
{
    let borrowed_source_config = source_config.borrow();
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d-%H%M%S");
    let uuid_string = Uuid::new_v4().to_string();
    let uuid = uuid_string.split('-').next().unwrap_or("backup");
    let source_name = get_source_name(borrowed_source_config);

    format!("{}-{}-{}.tar.gz", source_name, date_str, uuid)
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

pub fn get_backup_key(prefix: &str, db_type: &str, db_name: &str) -> String {
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d-%H%M%S");
    let uuid_string = Uuid::new_v4().to_string();
    let uuid = uuid_string.split('-').next().unwrap_or("backup");

    format!(
        "{}/{}/{}-{}-{}.gz",
        prefix, db_type, db_name, date_str, uuid
    )
}

pub fn format_timestamp(timestamp: &str) -> String {
    // The timestamp format can vary, but we'll try to handle common cases

    // If the timestamp is already in a standard format, try to parse it
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        return datetime.format("%Y-%m-%d %H:%M").to_string();
    }

    // Try parsing common timestamp formats
    // Format: YYYY-MM-DD-HHMMSS
    if timestamp.len() >= 17 && timestamp.contains('-') {
        if let Some(year_end) = timestamp.find('-') {
            if let Some(month_end) = timestamp[year_end + 1..]
                .find('-')
                .map(|pos| pos + year_end + 1)
            {
                if let Some(day_end) = timestamp[month_end + 1..]
                    .find('-')
                    .map(|pos| pos + month_end + 1)
                {
                    let year = &timestamp[..year_end];
                    let month = &timestamp[year_end + 1..month_end];
                    let day = &timestamp[month_end + 1..day_end];

                    // Handle the time part (HHMMSS)
                    let time_part = &timestamp[day_end + 1..];
                    if time_part.len() >= 4 {
                        let hour = &time_part[..2];
                        let minute = &time_part[2..4];

                        return format!("{}-{}-{} {}:{}", year, month, day, hour, minute);
                    }
                }
            }
        }
    }

    // If we couldn't parse the timestamp in a known format, return it as is
    timestamp.to_string()
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
