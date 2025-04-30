#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dotenv::dotenv;
    use log::LevelFilter;
    use serial_test::serial;
    use std::env;
    use tokio::sync::OnceCell;

    use crate::databases::{postgres::PostgreSQL, DbAdapter};

    struct ConnectionOptions {
        host: String,
        port: u16,
        user: String,
        password: String,
        db_name: String,
    }

    static LOGGER: OnceCell<()> = OnceCell::const_new();

    fn get_connection_options() -> Result<ConnectionOptions> {
        dotenv().ok();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;

        Ok(ConnectionOptions {
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: env::var("POSTGRESQL_PASSWORD").unwrap_or_default(),
            user: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            db_name: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            port,
        })
    }

    async fn initialize_test() {
        LOGGER
            .get_or_init(|| async {
                env_logger::Builder::new()
                    .filter_level(LevelFilter::Info)
                    .init();
            })
            .await;
    }

    fn get_postgresql() -> Result<PostgreSQL> {
        let options = get_connection_options()?;
        let postgresql = PostgreSQL::new(
            &options.db_name,
            &options.host,
            options.port,
            &options.user,
            Some(options.password.as_str()),
        );

        Ok(postgresql)
    }

    #[tokio::test]
    #[serial]
    async fn test_01_is_connected() {
        initialize_test().await;
        let postgresql = get_postgresql().expect("Failed to construct PostgreSQL adapter");
        let is_connected = postgresql
            .is_connected()
            .await
            .expect("Failed to check if MariaDB is connected");

        assert_eq!(is_connected, true);
    }

    #[tokio::test]
    #[serial]
    async fn test_02_dump() {
        initialize_test().await;
        let postgresql = get_postgresql().expect("Failed to construct PostgreSQL adapter");
        let bytes = postgresql.dump().await.expect("Unable to create a dump");

        assert!(bytes.len() > 0);
    }
}
