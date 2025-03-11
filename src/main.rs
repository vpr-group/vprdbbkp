use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use clap::Parser;
use log::{info, warn, LevelFilter};

mod cli;
mod config;
mod databases;
mod s3;
mod utils;

use cli::{Cli, Commands};
use databases::postgres;
use s3::upload_to_s3;
use utils::get_backup_key;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    let _ = dotenv::dotenv();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Set up logging
    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    // Set up AWS configuration
    let region_provider =
        RegionProviderChain::first_try(cli.region.map(aws_sdk_s3::config::Region::new))
            .or_default_provider()
            .or_else("us-east-1");

    // Disable SSL verification if requested
    if cli.no_verify_ssl {
        warn!("SSL verification disabled for S3 connections - this is not recommended for production use");
        // Set environment variable to disable SSL verification
        std::env::set_var("AWS_HTTPS_VERIFY", "0");
    }

    // Create the AWS config
    let aws_config = aws_config::from_env().region(region_provider).load().await;

    // Build S3 client configuration
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

    // Add custom endpoint if specified
    if let Some(endpoint) = &cli.endpoint {
        info!("Using custom S3 endpoint: {}", endpoint);
        s3_config_builder = s3_config_builder.endpoint_url(endpoint);

        // Force path style access for custom endpoints
        info!("Enabling path-style access for S3-compatible service");
        s3_config_builder = s3_config_builder.force_path_style(true);
    }

    // Add explicit credentials if provided
    if let (Some(access_key), Some(secret_key)) = (&cli.access_key, &cli.secret_key) {
        info!("Using explicitly provided S3 credentials");

        // Create static credentials
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None, // session token
            None, // expiry
            "explicit-credentials",
        );

        s3_config_builder = s3_config_builder.credentials_provider(credentials);
    }

    // Build the final S3 client with our configuration
    let s3_client = S3Client::from_conf(s3_config_builder.build());

    match cli.command {
        Commands::Postgres {
            database,
            host,
            port,
            username,
            password,
            compression,
            force_docker,
        } => {
            let backup_bytes = postgres::backup_postgres_with_options(
                &database,
                &host,
                port,
                &username,
                password.as_deref(),
                compression,
                force_docker,
            )
            .await?;

            let key = get_backup_key(&cli.prefix, "postgres", &database);
            upload_to_s3(
                &s3_client,
                &cli.bucket,
                &key,
                aws_sdk_s3::primitives::ByteStream::from(backup_bytes),
            )
            .await?;
        }
        Commands::Mysql {
            database,
            host,
            port,
            username,
            password,
            compression,
        } => {
            // let backup_bytes = mysql::backup_mysql(
            //     &database,
            //     &host,
            //     port,
            //     &username,
            //     password.as_deref(),
            //     compression,
            // )
            // .await?;

            // let key = get_backup_key(&cli.prefix, "mysql", &database);
            // upload_to_s3(
            //     &s3_client,
            //     &cli.bucket,
            //     &key,
            //     aws_sdk_s3::primitives::ByteStream::from(backup_bytes),
            // )
            // .await?;
        }
    }

    Ok(())
}
