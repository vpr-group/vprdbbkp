#[cfg(test)]
mod tests {
    use crate::databases::{mariadb::MariaDB, DbAdapter};
    use anyhow::Result;

    use dotenv::dotenv;
    use log::LevelFilter;
    use std::env;
    use tokio::sync::OnceCell;

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

        let port: u16 = env::var("MARIADB_PORT").unwrap_or("0".into()).parse()?;

        Ok(ConnectionOptions {
            host: env::var("MARIADB_HOST").unwrap_or_default(),
            password: env::var("MARIADB_PASSWORD").unwrap_or_default(),
            user: env::var("MARIADB_USERNAME").unwrap_or_default(),
            db_name: env::var("MARIADB_NAME").unwrap_or_default(),
            port,
        })
    }

    fn get_mariadb() -> Result<Box<dyn DbAdapter>> {
        let options = get_connection_options()?;
        let mariadb = MariaDB::new(
            &options.db_name,
            &options.host,
            options.port,
            &options.user,
            Some(options.password.as_str()),
        );

        Ok(Box::new(mariadb))
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

    #[tokio::test]
    async fn test_mariadb_is_connected() {
        initialize_test().await;
        let mariadb = get_mariadb().expect("Failed to construct MariaDB adapter");
        let is_connected = mariadb
            .is_connected()
            .await
            .expect("Failed to check if MariaDB is connected");

        assert_eq!(is_connected, true);
    }

    #[tokio::test]
    async fn test_mariadb_dump() {
        initialize_test().await;
        let mariadb = get_mariadb().expect("Failed to construct MariaDB adapter");
        let bytes = mariadb.dump(None).await.expect("Unable to create a dump");

        assert!(bytes.len() > 0);
    }
}
