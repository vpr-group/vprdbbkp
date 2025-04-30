use crate::databases::DbVersion;

use super::version::{PostgreSQLVersion, DEFAULT_POSTGRES_VERSION};
use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use flate2::read::GzDecoder;
use log::info;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;
use tempfile::Builder;
use tokio::fs;
use tokio::process::Command;

pub struct PgInstaller {
    version: PostgreSQLVersion,
    s3_bucket_url: String,
}

impl PgInstaller {
    pub fn new(version: PostgreSQLVersion) -> Self {
        PgInstaller { version, s3_bucket_url: "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/postgres".into() }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_POSTGRES_VERSION)
    }

    pub async fn install(&self, install_dir: PathBuf) -> Result<PathBuf> {
        info!("Installing PostgreSQL {}...", self.version.as_str());
        self.download_and_install(&install_dir).await?;

        if !Path::new(&install_dir).exists() {
            return Err(anyhow!("Installation failed at {}", install_dir.display()));
        }

        info!(
            "PostgreSQL {} successfully installed to {}",
            self.version.as_str(),
            install_dir.display()
        );

        Ok(install_dir)
    }

    async fn get_download_url(&self) -> Result<String> {
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
            os_info::Type::Arch => "linux",
            os_info::Type::Mint => "linux",
            os_info::Type::Windows => "windows",
            os_info::Type::Macos => "macos",
            _ => {
                return Err(anyhow!(
                    "Unsupported operating system: {:?}",
                    info.os_type()
                ))
            }
        };

        // Get system architecture using std
        let arch = std::env::consts::ARCH;

        // Map architecture to PostgreSQL download architecture
        let pg_arch = match arch {
            "x86_64" => "x86_64",
            "amd64" => "x86_64",
            "aarch64" => "arm64",
            "arm64" => "arm64",
            _ => return Err(anyhow!("Unsupported architecture: {}", arch)),
        };

        // Construct the S3 URL
        let download_url = format!(
            "{}/postgresql-{}-{}-{}.tar.gz",
            self.s3_bucket_url,
            self.version.as_str(),
            os,
            pg_arch
        );

        Ok(download_url)
    }

    async fn download_and_install(&self, install_dir: &PathBuf) -> Result<()> {
        // Create installation directory if it doesn't exist
        fs::create_dir_all(install_dir)
            .await
            .map_err(|_| anyhow!("Failed to create directory: {}", install_dir.display()))?;

        // Create temp directory for download
        let temp_dir = Builder::new()
            .prefix("postgres-download")
            .tempdir()
            .map_err(|_| anyhow!("Failed to create temporary directory"))?;

        let download_url = self.get_download_url().await?;
        let archive_path = temp_dir.path().join("postgresql.tar.gz");

        info!(
            "Downloading PostgreSQL {} from {}...",
            self.version.as_str(),
            download_url
        );

        // Download the archive using reqwest with async/await
        let response = reqwest::Client::new()
            .get(&download_url)
            .send()
            .await
            .map_err(|_| anyhow!("Failed to download PostgreSQL from {}", download_url))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download PostgreSQL: HTTP status {}",
                response.status()
            ));
        }

        // Get the response bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|_| anyhow!("Failed to read response body"))?;

        // Save the downloaded file
        fs::write(&archive_path, &bytes)
            .await
            .map_err(|_| anyhow!("Failed to write file: {}", archive_path.display()))?;

        info!(
            "Download complete. Extracting to {}...",
            install_dir.display()
        );

        // Extract the archive
        let tar_gz = fs::File::open(&archive_path).await.map_err(|_| {
            anyhow!(
                "Failed to open downloaded archive: {}",
                archive_path.display()
            )
        })?;

        let tar_gz_std = tar_gz.into_std().await;
        let tar = GzDecoder::new(tar_gz_std);
        let mut archive = Archive::new(tar);

        archive
            .unpack(install_dir)
            .map_err(|_| anyhow!("Failed to extract archive"))?;

        // Make binaries executable
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() {
            let mut read_dir = fs::read_dir(&bin_dir).await?;

            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                let _ = Command::new("chmod")
                    .arg("+x")
                    .arg(&path)
                    .output()
                    .await
                    .map_err(|_| anyhow!("Failed to make {} executable", path.display()));
            }
        }

        Ok(())
    }
}
