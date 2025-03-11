use anyhow::{Context, Result};
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use log::{debug, info};

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub key: String,
    pub size: i64,
    pub last_modified: DateTime<Utc>,
    pub db_name: String,
    pub backup_type: String,
    pub timestamp: String,
}

pub async fn upload_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    body: ByteStream,
) -> Result<()> {
    info!("Uploading to s3://{}/{}", bucket, key);

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await
        .context("Failed to upload to S3")?;

    info!("Upload completed successfully");
    Ok(())
}

/// List backups in the S3 bucket with optional filtering
pub async fn list_backups(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
    backup_type: Option<&str>,
    db_name: Option<&str>,
    limit: usize,
) -> Result<Vec<BackupInfo>> {
    info!("Listing backups in s3://{}/{}", bucket, prefix);
    debug!("sdéflkj");

    // Build the full prefix path based on filters
    let full_prefix = match (backup_type, db_name) {
        (Some(btype), Some(db)) => format!("{}/{}/{}", prefix, btype, db),
        (Some(btype), None) => format!("{}/{}", prefix, btype),
        (None, Some(_)) => {
            // Can't filter by db_name without backup_type
            format!("{}", prefix)
        }
        (None, None) => format!("{}", prefix),
    };

    let list_objects_output = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(&full_prefix)
        .send()
        .await
        .context("Failed to list objects in S3")?;

    let mut backups = Vec::new();

    if let Some(contents) = list_objects_output.contents() {
        for object in contents {
            let key = object.key().unwrap_or_default().to_string();

            // Skip non-backup files (like directory markers)
            if !key.ends_with(".gz") {
                continue;
            }

            let size = object.size();
            let last_modified = object
                .last_modified()
                .map(|dt| {
                    // Convert from AWS SDK DateTime to chrono::DateTime<Utc>
                    let timestamp = dt.as_secs_f64();
                    let secs = timestamp.floor() as i64;
                    let nsecs = ((timestamp - secs as f64) * 1_000_000_000.0) as u32;
                    chrono::DateTime::<Utc>::from_utc(
                        chrono::NaiveDateTime::from_timestamp_opt(secs, nsecs).unwrap(),
                        Utc,
                    )
                })
                .unwrap_or_else(|| Utc::now());

            // Parse backup info from key (format: prefix/type/db-timestamp-uuid.gz)
            let key_parts: Vec<&str> = key.split('/').collect();

            // Need at least prefix/type/filename
            if key_parts.len() >= 3 {
                let current_backup_type = key_parts[key_parts.len() - 2].to_string();

                // The last part is the filename, which should be db-timestamp-uuid.gz
                let filename = key_parts.last().unwrap_or(&"");

                // Split by hyphens to extract db_name and timestamp
                let filename_parts: Vec<&str> = filename.split('-').collect();

                if filename_parts.len() >= 2 {
                    let current_db_name = filename_parts[0].to_string();

                    // The timestamp is usually parts 1-6 (YYYY-MM-DD-HHMMSS)
                    // But we'll just take everything between the first hyphen and the last dot
                    let current_timestamp = if let Some(pos) = filename.find('-') {
                        if let Some(dot_pos) = filename.rfind('.') {
                            filename[pos + 1..dot_pos].to_string()
                        } else {
                            filename[pos + 1..].to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };

                    // Filter by backup_type and db_name if provided
                    if (backup_type.is_none()
                        || current_backup_type == backup_type.as_deref().unwrap_or(""))
                        && (db_name.is_none()
                            || current_db_name == db_name.as_deref().unwrap_or(""))
                    {
                        backups.push(BackupInfo {
                            key,
                            size,
                            last_modified,
                            db_name: current_db_name,
                            backup_type: current_backup_type,
                            timestamp: current_timestamp,
                        });
                    }
                }
            }
        }
    }

    // Sort by last_modified (newest first)
    backups.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    // Limit the number of results if requested
    if limit > 0 && backups.len() > limit {
        backups.truncate(limit);
    }

    Ok(backups)
}

/// Get the latest backup for a specific database and type
pub async fn get_latest_backup(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
    backup_type: &str,
    db_name: &str,
) -> Result<Option<BackupInfo>> {
    info!("Looking for latest backup of {}/{}", backup_type, db_name);

    let backups = list_backups(client, bucket, prefix, Some(backup_type), Some(db_name), 1).await?;

    Ok(backups.into_iter().next())
}

/// Download a backup from S3
pub async fn download_backup(client: &S3Client, bucket: &str, key: &str) -> Result<Bytes> {
    info!("Downloading backup from s3://{}/{}", bucket, key);

    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .context("Failed to download from S3")?;

    let data = resp.body.collect().await?;
    let bytes = data.into_bytes();

    info!("Downloaded {} bytes", bytes.len());
    Ok(bytes)
}

/// Group backups by database name and get latest for each
pub fn get_latest_backups_by_db(backups: &[BackupInfo]) -> Vec<&BackupInfo> {
    let mut latest_backups = Vec::new();
    let mut seen_dbs = std::collections::HashSet::new();

    for backup in backups {
        let db_key = format!("{}-{}", backup.backup_type, backup.db_name);

        if !seen_dbs.contains(&db_key) {
            seen_dbs.insert(db_key);
            latest_backups.push(backup);
        }
    }

    latest_backups
}
