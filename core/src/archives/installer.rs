use std::{env, fs, path::PathBuf};

use crate::{
    common::{get_binaries_base_path, get_db_name, get_version_name},
    databases::version::Version,
};
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use tokio::{fs::File, io::AsyncWriteExt, process::Command};

use super::DatabaseArchives;

const METADATA_URL: &str = "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/vprdbbkp/metadata.json";

pub struct ArchiveInstaller {
    database_version: Version,
}

impl ArchiveInstaller {
    pub fn new(database_version: Version) -> Self {
        ArchiveInstaller { database_version }
    }

    async fn get_database_archives_metadata(&self) -> Result<DatabaseArchives> {
        let response = reqwest::get(METADATA_URL).await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download: HTTP status {}", response.status()).into());
        }

        let archives: DatabaseArchives = response.json().await?;
        Ok(archives)
    }

    async fn get_archive_url(&self) -> Result<String> {
        let metadata = self.get_database_archives_metadata().await?;

        let (major_version, _, string_version) = match &self.database_version {
            Version::PostgreSQL(version) => (version.major, version.major, version.to_string()),
            Version::MySql(version) => (version.major, version.minor, version.to_string()),
        };

        let database_name = match self.database_version {
            Version::PostgreSQL(_) => "postgresql",
            Version::MySql(_) => "mysql",
        };

        let databases = match metadata
            .databases
            .iter()
            .find(|item| item.database == database_name)
        {
            Some(databases) => databases,
            None => {
                return Err(anyhow!(
                    "Database archive not available for: {}",
                    database_name
                ))
            }
        };

        let archive = match databases
            .archives
            .iter()
            .find(|item| item.version.major == major_version as u32)
        {
            Some(archive) => archive,
            None => return Err(anyhow!("Archive not found for version: {}", string_version)),
        };

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

        let url = match archive.platforms.get(format!("{}-{}", os, arch).as_str()) {
            Some(platform) => platform.url.clone(),
            None => {
                return Err(anyhow!(
                    "Unable to find an archive for platform: {}-{}",
                    os,
                    arch
                ))
            }
        };

        Ok(url)
    }

    async fn extract_tar_xz(archive_path: &PathBuf, destination: &PathBuf) -> Result<()> {
        debug!(
            "Extracting {} into {}",
            archive_path.display(),
            destination.display()
        );

        if !destination.exists() {
            fs::create_dir_all(destination)?;
        }

        let file = fs::File::open(archive_path)
            .with_context(|| format!("Failed to open archive file: {}", archive_path.display()))?;

        let xz_decoder = xz2::read::XzDecoder::new(file);
        let mut archive = tar::Archive::new(xz_decoder);

        archive.unpack(destination).with_context(|| {
            format!(
                "Failed to extract tar.xz archive to {}",
                destination.display()
            )
        })?;

        // Handle file permissions for bin directory
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

        #[cfg(target_os = "macos")]
        {
            debug!("Running on macOS, removing quarantine attribute...");
            let output = Command::new("xattr")
                .arg("-dr")
                .arg("com.apple.quarantine")
                .arg(destination)
                .output()
                .await?;

            if output.status.success() {
                debug!("Successfully removed quarantine attribute");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                log::error!("Error removing quarantine attribute: {}", error);

                if !error.contains("No such xattr") {
                    return Err(anyhow!("xattr command failed: {}", error));
                }
            }
        }

        Ok(())
    }

    async fn extract_zip(archive_path: &PathBuf, destination: &PathBuf) -> Result<()> {
        debug!(
            "Extracting {} into {}",
            archive_path.display(),
            destination.display()
        );

        let status = Command::new("powershell")
            .arg("-Command")
            .arg(&format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}'",
                archive_path.display(),
                destination.display()
            ))
            .status()
            .await
            .with_context(|| "Failed to execute PowerShell to extract zip archive")?;

        if !status.success() {
            return Err(anyhow!("Failed to extract zip archive"));
        }

        Ok(())
    }

    pub async fn download_and_install(&self) -> Result<PathBuf> {
        let archive_url = self.get_archive_url().await?;
        let binaries_base_bath = get_binaries_base_path(&self.database_version);

        if !binaries_base_bath.exists() {
            fs::create_dir_all(&binaries_base_bath).with_context(|| {
                format!(
                    "Failed to create directory: {}",
                    binaries_base_bath.display()
                )
            })?;
        }

        info!("Downloading archive from {}", archive_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&archive_url)
            .send()
            .await
            .with_context(|| format!("Failed to download archive from {}", archive_url))?;

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

        let db_name = get_db_name(&self.database_version);
        let version_name = get_version_name(&self.database_version);
        let archive_path = temp_dir.join(format!("{}-{}", db_name, version_name));

        let mut file = File::create(&archive_path)
            .await
            .with_context(|| format!("Failed to create file: {}", archive_path.display()))?;

        file.write_all(&content)
            .await
            .with_context(|| format!("Failed to write to file: {}", archive_path.display()))?;

        file.sync_all().await?;

        if cfg!(target_os = "windows") {
            Self::extract_zip(&archive_path, &binaries_base_bath).await?;
        } else {
            Self::extract_tar_xz(&archive_path, &binaries_base_bath).await?;
        }

        tokio::fs::remove_file(archive_path).await.ok();

        Ok(binaries_base_bath)
    }
}
