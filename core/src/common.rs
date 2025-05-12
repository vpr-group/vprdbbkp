use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use dirs::cache_dir;
use log::info;
use regex::Regex;
use std::{
    borrow::Borrow,
    env, fs,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncWriteExt};
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

fn get_db_name(version: &Version) -> String {
    match version {
        Version::PostgreSQL(_) => "postgresql".into(),
        Version::MySql(_) => "mysql".into(),
    }
}

fn get_version_name(version: &Version) -> String {
    match version {
        Version::PostgreSQL(version) => version.to_string(),
        Version::MySql(version) => version.to_string(),
    }
}

pub fn get_binary_archive_url(version: &Version) -> Result<String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow!("Unsupported architecture"));
    };

    let archive_extension = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };

    let db_name = get_db_name(&version);
    let version_name = get_version_name(&version);

    let base_bucket_url =
        "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/vprdbbkp";

    let archive_name = format!(
        "{}-{}-{}.{}",
        db_name, version_name, arch, archive_extension
    );

    let url = format!("{}/{}/{}", base_bucket_url, os, archive_name);

    Ok(url)
}

async fn extract_zip(archive_path: &PathBuf, destination: &PathBuf) -> Result<()> {
    use std::process::Command as StdCommand;

    let status = StdCommand::new("powershell")
        .arg("-Command")
        .arg(&format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}'",
            archive_path.display(),
            destination.display()
        ))
        .status()
        .with_context(|| "Failed to execute PowerShell to extract zip archive")?;

    if !status.success() {
        return Err(anyhow!("Failed to extract zip archive"));
    }

    Ok(())
}

async fn extract_tar_gz(archive_path: &PathBuf, destination: &PathBuf) -> Result<()> {
    use std::process::Command as StdCommand;

    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    let status = StdCommand::new("tar")
        .arg("-xzf")
        .arg(archive_path)
        .arg("-C")
        .arg(destination)
        .status()
        .with_context(|| "Failed to execute tar to extract archive")?;

    if !status.success() {
        return Err(anyhow!("Failed to extract tar.gz archive"));
    }

    let bin_dir = destination.join("bin");
    if bin_dir.exists() {
        let entries = fs::read_dir(&bin_dir)?;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    #[cfg(not(target_os = "windows"))]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let metadata = fs::metadata(&path)?;
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755); // rwxr-xr-x
                        fs::set_permissions(&path, perms)?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn download_and_install_binaries(version: &Version) -> Result<PathBuf> {
    let base_path = get_binaries_base_path(&version);

    if !base_path.exists() {
        fs::create_dir_all(&base_path)
            .with_context(|| format!("Failed to create directory: {}", base_path.display()))?;
    }

    let url = get_binary_archive_url(&version)?;

    info!("Downloading archive from {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to download archive from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download archive, server returned: {}",
            response.status()
        ));
    }

    let content = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    let temp_dir = env::temp_dir();

    let db_name = get_db_name(&version);
    let version_name = get_version_name(&version);
    let archive_path = temp_dir.join(format!("{}-{}", db_name, version_name));

    let mut file = File::create(&archive_path)
        .await
        .with_context(|| format!("Failed to create file: {}", archive_path.display()))?;

    file.write_all(&content)
        .await
        .with_context(|| format!("Failed to write to file: {}", archive_path.display()))?;

    file.sync_all().await?;

    if cfg!(target_os = "windows") {
        extract_zip(&archive_path, &base_path).await?;
    } else {
        extract_tar_gz(&archive_path, &base_path).await?;
    }

    tokio::fs::remove_file(archive_path).await.ok();

    Ok(base_path)
}

pub fn get_source_name<B>(source_config: B) -> String
where
    B: Borrow<DatabaseConfig>,
{
    let borrowed_source_config: &DatabaseConfig = source_config.borrow();
    format!(
        "{}-{}",
        slugify(&borrowed_source_config.name),
        slugify(&borrowed_source_config.database),
    )
}

pub fn get_filename<B>(source_config: B) -> String
where
    B: Borrow<DatabaseConfig>,
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
