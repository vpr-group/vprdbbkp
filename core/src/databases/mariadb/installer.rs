use std::path::{Path, PathBuf};

use crate::{
    common::{copy_dir_all, get_arch, get_os},
    databases::DbVersion,
};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use log::info;
use tar::Archive;
use tempfile::Builder;
use tokio::{fs, process::Command};

use super::version::{MariaDBVersion, DEFAULT_MARIADB_VERSION};

pub struct MariaDBInstaller {
    version: MariaDBVersion,
    s3_bucket_url: String,
}

impl MariaDBInstaller {
    pub fn new(version: MariaDBVersion) -> Self {
        MariaDBInstaller { version, s3_bucket_url: "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/mariadb".into() }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_MARIADB_VERSION)
    }

    async fn get_download_url(&self) -> Result<String> {
        let os = get_os()?;
        let arch = get_arch()?;

        // Construct the S3 URL
        let download_url = format!(
            "{}/mariadb-{}-{}-{}.tar.gz",
            self.s3_bucket_url,
            self.version.as_str(),
            os,
            arch
        );

        Ok(download_url)
    }

    async fn download(&self) -> Result<Bytes> {
        let download_url = self.get_download_url().await?;

        println!("{}", download_url);

        info!(
            "Downloading MariaDB {} from {}...",
            self.version.as_str(),
            download_url
        );

        let response = reqwest::Client::new()
            .get(&download_url)
            .send()
            .await
            .map_err(|_| anyhow!("Failed to download MariaDB from {}", download_url))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download MariaDB: HTTP status {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|_| anyhow!("Failed to read response body"))?;

        Ok(bytes)
    }

    async fn extract(
        &self,
        install_dir: &PathBuf,
        bytes: &Bytes,
        new_folder_name: &str,
    ) -> Result<()> {
        let temp_dir = Builder::new()
            .prefix("mariadb-download")
            .tempdir()
            .map_err(|_| anyhow!("Failed to create temporary directory"))?;

        let archive_path = temp_dir.path().join("mariadb.tar.gz");

        fs::write(&archive_path, &bytes)
            .await
            .map_err(|_| anyhow!("Failed to write file: {}", archive_path.display()))?;

        let tar_gz = fs::File::open(&archive_path).await.map_err(|_| {
            anyhow!(
                "Failed to open downloaded archive: {}",
                archive_path.display()
            )
        })?;

        let tar_gz_std = tar_gz.into_std().await;
        let tar = GzDecoder::new(tar_gz_std);
        let mut archive = Archive::new(tar);

        // Extract to the temporary directory first
        archive
            .unpack(temp_dir.path())
            .map_err(|_| anyhow!("Failed to extract archive"))?;

        // Find the extracted top-level directory
        let mut extracted_dir = None;
        let mut dir_entries = fs::read_dir(temp_dir.path())
            .await
            .map_err(|_| anyhow!("Failed to read temporary directory"))?;

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|_| anyhow!("Failed to read directory entry"))?
        {
            if entry
                .file_type()
                .await
                .map_err(|_| anyhow!("Failed to get file type"))?
                .is_dir()
            {
                extracted_dir = Some(entry.path());
                break;
            }
        }

        let extracted_dir =
            extracted_dir.ok_or_else(|| anyhow!("No directory found in extracted archive"))?;

        // Create the target directory with the new name
        let target_dir = install_dir.join(new_folder_name);

        // Create the directory if it doesn't exist
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)
                .await
                .map_err(|_| anyhow!("Failed to create target directory"))?;
        }

        // Copy all files from extracted dir to renamed target dir
        copy_dir_all(&extracted_dir, &target_dir)?;

        Ok(())
    }

    async fn download_and_install(&self, install_dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(install_dir)
            .await
            .map_err(|_| anyhow!("Failed to create directory: {}", install_dir.display()))?;

        let bytes = self.download().await?;

        info!(
            "Download complete. Extracting to {}...",
            install_dir.display()
        );

        self.extract(install_dir, &bytes, self.version.as_str())
            .await?;

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

    pub async fn install(&self, install_dir: PathBuf) -> Result<PathBuf> {
        info!("Installing MariaDB {}...", self.version.as_str());

        self.download_and_install(&install_dir).await?;

        if !Path::new(&install_dir).exists() {
            return Err(anyhow!("Installation failed at {}", install_dir.display()));
        }

        info!(
            "MariaDB {} successfully installed to {}",
            self.version.as_str(),
            install_dir.display()
        );

        Ok(install_dir)
    }
}
