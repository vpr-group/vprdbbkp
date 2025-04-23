use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::borrow::Borrow;
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

pub fn get_filename<B>(backup_source_config: B) -> String
where
    B: Borrow<SourceConfig>,
{
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d-%H%M%S");
    let uuid_string = Uuid::new_v4().to_string();
    let uuid = uuid_string.split('-').next().unwrap_or("backup");

    match backup_source_config.borrow() {
        SourceConfig::PG(config) => format!(
            "{}-{}-{}-{}.gz",
            slugify(&config.name),
            slugify(&config.database),
            date_str,
            uuid
        ),
        SourceConfig::MariaDB(config) => format!(
            "{}-{}-{}-{}.gz",
            slugify(&config.name),
            slugify(&config.database),
            date_str,
            uuid
        ),
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
