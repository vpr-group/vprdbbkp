use crate::commands::S3StorageProvider;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use log::info;
use std::borrow::Borrow;

pub async fn get_s3_client<T>(storage_provider: T) -> Result<S3Client, String>
where
    T: Borrow<S3StorageProvider>,
{
    let storage_provider: &S3StorageProvider = storage_provider.borrow();

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
    Ok(s3_client)
}
