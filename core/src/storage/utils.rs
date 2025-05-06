use anyhow::{anyhow, Result};
use reqwest::Url;

pub fn parse_s3_location(location: &str) -> Result<(String, String)> {
    if let Some(bucket) = location.strip_prefix("s3://") {
        return Ok(("https://s3.amazonaws.com".to_string(), bucket.to_string()));
    }

    // If it's a URL, extract the endpoint and bucket
    if location.starts_with("http://") || location.starts_with("https://") {
        let url = Url::parse(location).map_err(|e| anyhow!("Invalid S3 URL: {}", e))?;

        let endpoint = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));

        // The bucket is the first path segment
        let bucket = url
            .path()
            .trim_start_matches('/')
            .split('/')
            .next()
            .ok_or_else(|| anyhow!("Missing bucket in S3 URL".to_string()))?
            .to_string();

        return Ok((endpoint, bucket));
    }

    Ok(("https://s3.amazonaws.com".to_string(), location.to_string()))
}
