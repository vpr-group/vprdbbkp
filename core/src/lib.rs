// vprs3bkp-core/src/lib.rs
pub mod databases;
pub mod folders;
pub mod platform;
pub mod s3;
pub mod storage;
pub mod utils;

// Re-export commonly used types and functions
pub use databases::{
    mysql::{self, backup_mysql, backup_mysql_with_options},
    postgres::{self, backup_postgres, backup_postgres_with_options, pg_restore::restore_postgres},
};
pub use folders::{backup_folder, BackupStats};
pub use s3::{
    download_backup, get_latest_backup, get_latest_backups_by_db, list_backups, upload_to_s3,
    BackupInfo,
};
pub use utils::{format_timestamp, get_backup_key};
