#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::cargo::CommandCargoExt;
    use dotenv::dotenv;
    use regex::Regex;
    use std::env;
    use std::process::Command;

    struct ConnectionOptions {
        host: String,
        port: u16,
        user: String,
        password: String,
        db_name: String,
    }

    fn get_postgresql_connection_options() -> Result<ConnectionOptions> {
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

    fn get_mariadb_connection_options() -> Result<ConnectionOptions> {
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

    fn extract_filename(text: &str) -> Option<String> {
        let re = Regex::new(r"\s+([a-zA-Z0-9_-]+-\d{4}-\d{2}-\d{2}-\d{6}-[a-z0-9]+\.gz)").unwrap();

        if let Some(cap) = re.captures(text) {
            return Some(cap[1].to_string());
        }

        None
    }

    #[test]
    fn test_01_backup_list_restore_postgresql() {
        let options = get_postgresql_connection_options().expect("Failed to get postgres options");
        let mut cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let backup_output = cmd
            .arg("backup")
            .arg(format!("--database-type=postgresql"))
            .arg(format!("--database={}", options.db_name))
            .arg(format!("--host={}", options.host))
            .arg(format!("--username={}", options.user))
            .arg(format!("--password={}", options.password))
            .arg(format!("--port={}", options.port))
            .arg(format!("--storage-type=local"))
            .arg(format!("--location=./test-backups-postgresql"))
            .output()
            .expect("Failed to execute command");

        assert!(backup_output.status.success());
        assert!(String::from_utf8_lossy(&backup_output.stdout)
            .contains("Backup completed successfully"));

        cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let list_output = cmd
            .arg("list")
            .arg("--latest-only")
            .arg(format!("--storage-type=local"))
            .arg(format!("--location=./test-backups-postgresql"))
            .output()
            .expect("Failed to execute command");

        let string_output = String::from_utf8_lossy(&list_output.stdout);
        let file_name = extract_filename(&string_output.to_owned());

        assert_eq!(file_name.is_some(), true);

        let file_name = file_name.unwrap();

        cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let restore_output = cmd
            .arg("restore")
            .arg(format!("--name={}", file_name))
            .arg(format!("--drop-database"))
            .arg(format!("--database-type=postgresql"))
            .arg(format!("--database={}", options.db_name))
            .arg(format!("--host={}", options.host))
            .arg(format!("--username={}", options.user))
            .arg(format!("--password={}", options.password))
            .arg(format!("--port={}", options.port))
            .arg(format!("--storage-type=local"))
            .arg(format!("--location=./test-backups-postgresql"))
            .output()
            .expect("Failed to execute command");

        assert!(String::from_utf8_lossy(&restore_output.stdout)
            .contains("Restore completed successfully"));
    }

    #[test]
    fn test_02_backup_list_restore_mariadb() {
        let options = get_mariadb_connection_options().expect("Failed to get mariadb options");
        let mut cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let backup_output = cmd
            .arg("backup")
            .arg(format!("--source-type=mariadb"))
            .arg(format!("--database={}", options.db_name))
            .arg(format!("--host={}", options.host))
            .arg(format!("--username={}", options.user))
            .arg(format!("--password={}", options.password))
            .arg(format!("--port={}", options.port))
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-mariadb"))
            .output()
            .expect("Failed to execute command");

        assert!(backup_output.status.success());
        assert!(String::from_utf8_lossy(&backup_output.stdout)
            .contains("Backup completed successfully"));

        cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let list_output = cmd
            .arg("list")
            .arg("--latest-only")
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-mariadb"))
            .output()
            .expect("Failed to execute command");

        let string_output = String::from_utf8_lossy(&list_output.stdout);
        let file_name = extract_filename(&string_output.to_owned());

        assert_eq!(file_name.is_some(), true);

        let file_name = file_name.unwrap();

        cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let restore_output = cmd
            .arg("restore")
            .arg(format!("--filename={}", file_name))
            .arg(format!("--drop-database=true"))
            .arg(format!("--source-type=mariadb"))
            .arg(format!("--database={}", options.db_name))
            .arg(format!("--host={}", options.host))
            .arg(format!("--username={}", options.user))
            .arg(format!("--password={}", options.password))
            .arg(format!("--port={}", options.port))
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-mariadb"))
            .output()
            .expect("Failed to execute command");

        assert!(String::from_utf8_lossy(&restore_output.stdout)
            .contains("Restore completed successfully"));
    }
}
