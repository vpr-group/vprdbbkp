#[cfg(test)]
mod postgresql_tests {
    use anyhow::Result;
    use dotenv::dotenv;
    use log::LevelFilter;
    use serial_test::serial;
    use std::env;
    use tokio::{process::Command, sync::OnceCell};

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
                let _ = env_logger::Builder::new()
                    .filter_level(LevelFilter::Info)
                    .try_init();
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

    async fn get_postgres_connection() -> Result<Command> {
        let options = get_connection_options()?;
        let postgres = get_postgresql()?;
        let tools = postgres.get_tools().await?;
        let connection = tools
            .get_connection(
                &options.db_name,
                &options.host,
                options.port,
                &options.user,
                Some(options.password.as_str()),
            )
            .await?;

        Ok(connection)
    }

    async fn drop_and_restore_db() -> Result<()> {
        let options = get_connection_options()?;
        let postgres = get_postgresql()?;
        let tools = postgres.get_tools().await?;
        tools
            .drop_and_recreate_database(
                &options.db_name,
                &options.host,
                options.port,
                &options.user,
                Some(options.password.as_str()),
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_01_is_connected() {
        initialize_test().await;
        let postgresql = get_postgresql().expect("Failed to construct PostgreSQL adapter");
        let is_connected = postgresql
            .is_connected()
            .await
            .expect("Failed to check if PostgreSQL is connected");

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

    #[tokio::test]
    #[serial]
    async fn test_03_restore() {
        initialize_test().await;
        let postgresql = get_postgresql().expect("Failed to construct PostgreSQL adapter");
        let mut connection = get_postgres_connection()
            .await
            .expect("Failed to get connection command");

        let test_table_name = format!("test_restore_{}", chrono::Utc::now().timestamp());

        let create_table_cmd = connection
            .arg(format!(
                "CREATE TABLE {} (id INT PRIMARY KEY, value VARCHAR(255))",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute create table command");

        if !create_table_cmd.status.success() {
            panic!(
                "Failed to create test table: {}",
                String::from_utf8_lossy(&create_table_cmd.stderr)
            );
        }

        connection = get_postgres_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let insert_data_cmd = connection
            .arg(format!(
                "INSERT INTO {} VALUES (1, 'test_value')",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute insert command");

        if !insert_data_cmd.status.success() {
            panic!(
                "Failed to insert test data: {}",
                String::from_utf8_lossy(&insert_data_cmd.stderr)
            );
        }

        connection = get_postgres_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );

        let bytes = postgresql.dump().await.expect("Unable to create a dump");

        assert!(bytes.len() > 0);

        drop_and_restore_db()
            .await
            .expect("Failed to drop and restore the database");

        connection = get_postgres_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        // Expect the query to fail as the table has been dropped
        assert_eq!(select_data_cmd.status.success(), false);

        postgresql
            .restore(bytes, true)
            .await
            .expect("Failed to restore database");

        connection = get_postgres_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );
    }
}
