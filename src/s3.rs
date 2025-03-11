use anyhow::{Context, Result};
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use log::info;

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
