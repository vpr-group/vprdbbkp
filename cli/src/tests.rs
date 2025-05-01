#[cfg(test)]
mod tests {
    use assert_cmd::cargo::CommandCargoExt;
    use regex::Regex;
    use std::process::Command;

    fn extract_filename(text: &str) -> Option<String> {
        let re =
            Regex::new(r"\s+([a-zA-Z0-9_-]+-\d{4}-\d{2}-\d{2}-\d{6}-[a-z0-9]+\.tar\.gz)").unwrap();

        if let Some(cap) = re.captures(text) {
            return Some(cap[1].to_string());
        }

        None
    }

    #[test]
    fn test_01_backup_list_restore_postgresql() {
        let mut cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let backup_output = cmd
            .arg("backup")
            .arg(format!("--source-type=postgres"))
            .arg(format!("--database=postgres_database"))
            .arg(format!("--host=localhost"))
            .arg(format!("--username=postgres"))
            .arg(format!("--password=postgres_password"))
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-postgresql"))
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
            .arg(format!("--root=./test-backups-postgresql"))
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
            .arg(format!("--source-type=postgres"))
            .arg(format!("--database=postgres_database"))
            .arg(format!("--host=localhost"))
            .arg(format!("--username=postgres"))
            .arg(format!("--password=postgres_password"))
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-postgresql"))
            .output()
            .expect("Failed to execute command");

        assert!(String::from_utf8_lossy(&restore_output.stdout)
            .contains("Restore completed successfully"));
    }

    #[test]
    fn test_02_backup_list_restore_mariadb() {
        let mut cmd = Command::cargo_bin("cli").expect("Failed to get cli command");

        let backup_output = cmd
            .arg("backup")
            .arg(format!("--source-type=mariadb"))
            .arg(format!("--database=mariadb_database"))
            .arg(format!("--host=localhost"))
            .arg(format!("--username=root"))
            .arg(format!("--port=3306"))
            .arg(format!("--password=mariadb_root_password"))
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
            .arg(format!("--database=mariadb_database"))
            .arg(format!("--port=3306"))
            .arg(format!("--host=localhost"))
            .arg(format!("--username=root"))
            .arg(format!("--password=mariadb_root_password"))
            .arg(format!("--storage-type=local"))
            .arg(format!("--root=./test-backups-mariadb"))
            .output()
            .expect("Failed to execute command");

        println!("{}", String::from_utf8_lossy(&restore_output.stdout));
        println!("{}", String::from_utf8_lossy(&restore_output.stderr));

        assert!(String::from_utf8_lossy(&restore_output.stdout)
            .contains("Restore completed successfully"));
    }
}
