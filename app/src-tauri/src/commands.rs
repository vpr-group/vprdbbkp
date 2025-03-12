use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use log::info;
use serde::{Deserialize, Serialize};
use vprs3bkp_core::BackupInfo;

// Define types that match your TypeScript interfaces
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3StorageProvider {
    #[serde(rename = "type")]
    provider_type: String,
    region: Option<String>,
    bucket: String,
    endpoint: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackupListItem {
    name: String,
    size: u64,
    timestamp: String,
    status: String,
}

#[tauri::command]
pub async fn list_backups(storage_provider: S3StorageProvider) -> Result<Vec<BackupInfo>, String> {
    // Ensure we're dealing with an S3 provider
    if storage_provider.provider_type != "s3" {
        return Err("Only S3 storage providers are supported".to_string());
    }

    info!("{:?}", storage_provider);

    // Set up AWS configuration
    let region_provider = RegionProviderChain::first_try(
        storage_provider
            .region
            .clone()
            .map(aws_sdk_s3::config::Region::new),
    )
    .or_default_provider()
    .or_else("us-east-1");

    // Create the AWS config
    let aws_config = aws_config::from_env().region(region_provider).load().await;

    // Build S3 client configuration
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

    // Add custom endpoint if specified
    if let Some(endpoint) = &storage_provider.endpoint {
        let trimmed_endpoint = endpoint.trim();

        info!("Using custom S3 endpoint: {}", trimmed_endpoint);
        s3_config_builder = s3_config_builder.endpoint_url(trimmed_endpoint);

        // Force path style access for custom endpoints
        info!("Enabling path-style access for S3-compatible service");
        s3_config_builder = s3_config_builder.force_path_style(true);
    }

    // Add explicit credentials if provided
    if let (Some(access_key), Some(secret_key)) =
        (&storage_provider.access_key, &storage_provider.secret_key)
    {
        info!("Using explicitly provided S3 credentials");

        // Create static credentials
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key.trim(),
            secret_key.trim(),
            None, // session token
            None, // expiry
            "explicit-credentials",
        );

        s3_config_builder = s3_config_builder.credentials_provider(credentials);
    }

    // Build the final S3 client with our configuration
    let s3_client = S3Client::from_conf(s3_config_builder.build());

    let backups = vprs3bkp_core::list_backups(
        &s3_client,
        &storage_provider.bucket.trim(),
        "",
        None,
        None,
        10,
    )
    .await
    .map_err(|e| format!("Failed to list objects: {}", e))?;

    Ok(backups)
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
