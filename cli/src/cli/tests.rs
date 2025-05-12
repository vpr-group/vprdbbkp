#[cfg(test)]
mod cli_test {
    use vprs3bkp_core::databases::ConnectionType;

    use crate::cli::{
        database_config_from_cli, storage_from_cli, DatabaseArgs, SshArgs, StorageArgs,
    };

    #[test]
    fn test_01_parse_backup_command() {
        let database_args = DatabaseArgs {
            database_type: "postgresql".into(),
            database: "test".into(),
            host: "localhost".into(),
            port: 5432,
            username: "username".into(),
            password: Some("password".into()),
            ssh: Some(SshArgs {
                ssh_host: Some("ssh_host".into()),
                ssh_username: Some("ssh_username".into()),
                ssh_key_path: Some("ssh_key_path".into()),
            }),
        };

        let database_config =
            database_config_from_cli(&database_args).expect("Failed to parse database args");

        assert_eq!(database_config.connection_type, ConnectionType::PostgreSql);
        assert_eq!(database_config.database, "test");
        assert_eq!(database_config.host, "localhost");
        assert_eq!(database_config.port, 5432);
        assert_eq!(database_config.username, "username");
        assert_eq!(database_config.password.clone().unwrap(), "password");

        let ssh_config = database_config.ssh_tunnel.clone().unwrap();

        assert_eq!(ssh_config.host, "ssh_host");
        assert_eq!(ssh_config.username, "ssh_username");
    }

    #[test]
    fn test_02_parse_storage_config() {
        let storage_args = StorageArgs {
            storage_type: "s3".into(),
            storage_name: "test".into(),
            prefix: Some("".into()),
            bucket: Some("bucket".into()),
            region: Some("region".into()),
            endpoint: Some("endpoint".into()),
            access_key: Some("access_key".into()),
            secret_key: Some("access_key".into()),
            root: None,
        };

        let storage_config = storage_from_cli(&storage_args);

        println!("{:?}", storage_config);
    }
}
