use crate::databases::connection::DatabaseConnection;
use anyhow::Result;

pub struct BackupEngine {
    database_connection: DatabaseConnection,
}

impl BackupEngine {
    fn new(database_connection: DatabaseConnection) -> Self {
        Self {
            database_connection,
        }
    }

    // pub async backup(&self) -> Result<()> {

    // }
}
