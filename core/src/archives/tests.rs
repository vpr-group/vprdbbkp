#[cfg(test)]
mod archives_tests {
    use crate::{
        archives::installer::ArchiveInstaller,
        databases::{
            mysql::version::MySqlVersion, postgres::version::PostgreSQLVersion, version::Version,
        },
    };
    use dotenv::dotenv;

    fn initialize() {
        dotenv().ok();
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    }

    #[tokio::test]
    async fn test_01_install_postgres() {
        initialize();
        let archive_installer = ArchiveInstaller::new(Version::PostgreSQL(PostgreSQLVersion {
            major: 17,
            minor: 3,
        }));

        let path = archive_installer
            .download_and_install()
            .await
            .expect("Failed to download and install");

        assert!(path.to_string_lossy().contains("postgresql/17"));
    }

    #[tokio::test]
    async fn test_02_install_mysql() {
        initialize();
        let archive_installer = ArchiveInstaller::new(Version::MySql(MySqlVersion {
            major: 9,
            minor: 3,
            patch: 0,
        }));

        let path = archive_installer
            .download_and_install()
            .await
            .expect("Failed to download and install");

        assert!(path.to_string_lossy().contains("mysql/9"));
    }
}
