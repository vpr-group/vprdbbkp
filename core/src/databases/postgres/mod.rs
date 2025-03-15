use anyhow::Result;
use bytes::Bytes;
use pg_tools::PgTools;

pub mod pg_tools;
pub mod pg_versions;

pub async fn is_postgres_connected(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
) -> Result<bool> {
    let pg_tools = PgTools::default()?;

    let is_connected = pg_tools
        .is_postgres_connected(database, host, port, username, password)
        .await?;

    Ok(is_connected)
}

pub async fn backup_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: Option<u8>,
) -> Result<Bytes> {
    let mut pg_tools = PgTools::default()?;

    let version = pg_tools
        .get_postgres_version(database, host, port, username, password)
        .await?;

    // Set the correct PostrgreSQL target version
    pg_tools = PgTools::new(version)?;

    let output = pg_tools
        .dump(database, host, port, username, password, compression)
        .await?;

    Ok(output)
}

pub async fn restore_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    dump_data: Bytes,
    compressed: bool,
) -> Result<()> {
    let mut pg_tools = PgTools::default()?;

    let version = pg_tools
        .get_postgres_version(database, host, port, username, password)
        .await?;

    // Set the correct PostrgreSQL target version
    pg_tools = PgTools::new(version)?;

    pg_tools
        .restore(
            database, host, port, username, password, dump_data, compressed,
        )
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        let is_connected =
            is_postgres_connected("api", "localhost", 5432, "postgres", Some("postgres"))
                .await
                .expect("Unable to check database connection");

        let is_not_connected =
            is_postgres_connected("random", "localhost", 5432, "postgres", Some("postgres"))
                .await
                .expect("Unable to check database connection");

        assert!(is_connected);
        assert_eq!(is_not_connected, false);
    }

    #[tokio::test]
    async fn test_backup() {
        let backup = backup_postgres("api", "localhost", 5432, "postgres", Some("postgres"), None)
            .await
            .expect("Unable to backup database");

        let compressed_backup = backup_postgres(
            "api",
            "localhost",
            5432,
            "postgres",
            Some("postgres"),
            Some(9),
        )
        .await
        .expect("Unable to backup database");

        assert!(backup.len() > 0);
        assert!(compressed_backup.len() > 0);
        assert!(compressed_backup.len() < backup.len());
    }
}
