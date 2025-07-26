#[cfg(test)]
pub mod test_utils {
    use std::{env, time::Duration};

    use anyhow::Result;
    use dotenv::dotenv;
    use sqlx::{
        mysql::{MySqlConnectOptions, MySqlPoolOptions},
        postgres::{PgConnectOptions, PgPoolOptions},
        MySql, Pool, Postgres,
    };
    use tempfile::tempdir;

    use crate::{
        databases::{postgres::connection::PostgreSqlConnection, ConnectionType, DatabaseConfig},
        storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider},
    };

    pub fn initialize_test() {
        dotenv().ok();
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    }

    pub async fn get_postgresql_pool() -> Result<Pool<Postgres>> {
        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();
        let host: String = env::var("POSTGRESQL_HOST").unwrap_or_default();
        let username: String = env::var("POSTGRESQL_USERNAME").unwrap_or_default();
        let database: String = env::var("POSTGRESQL_NAME").unwrap_or_default();

        let connect_options = PgConnectOptions::new()
            .host(&host)
            .username(&username)
            .database(&database)
            .password(&password)
            .port(port);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .connect_with(connect_options)
            .await?;

        Ok(pool)
    }

    pub async fn get_mysql_pool() -> Result<Pool<MySql>> {
        let port: u16 = env::var("MYSQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("MYSQL_PASSWORD").unwrap_or_default();
        let host: String = env::var("MYSQL_HOST").unwrap_or_default();
        let username: String = env::var("MYSQL_USERNAME").unwrap_or_default();
        let database: String = env::var("MYSQL_NAME").unwrap_or_default();

        let connect_options = MySqlConnectOptions::new()
            .host(&host)
            .username(&username)
            .database(&database)
            .password(&password)
            .port(port);

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .connect_with(connect_options)
            .await?;

        Ok(pool)
    }

    pub async fn get_postgresql_connection(admin_connection: bool) -> Result<PostgreSqlConnection> {
        initialize_test();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();
        let connection = PostgreSqlConnection::new(DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSql,
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: Some(password),
            username: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            database: if admin_connection {
                "postgres".into()
            } else {
                env::var("POSTGRESQL_NAME").unwrap_or_default()
            },
            port,
            ssh_tunnel: None,
        })
        .await?;

        Ok(connection)
    }

    pub fn get_local_provider() -> Result<StorageProvider> {
        let temp_path = tempdir()?;
        let config = StorageConfig::Local(LocalStorageConfig {
            id: "test".into(),
            name: "local".into(),
            location: temp_path.path().to_str().unwrap().to_string(),
        });
        let provider = StorageProvider::new(config)?;
        Ok(provider)
    }

    pub fn get_s3_provider() -> Result<StorageProvider> {
        let location = format!("s3_provider_test_{}", chrono::Utc::now().timestamp());

        let endpoint = env::var("S3_ENDPOINT")
            .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string());

        let config = StorageConfig::S3(S3StorageConfig {
            id: "test".into(),
            name: "s3".into(),
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: Some(endpoint),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            location,
        });

        let provider = StorageProvider::new(config)?;

        Ok(provider)
    }
}
