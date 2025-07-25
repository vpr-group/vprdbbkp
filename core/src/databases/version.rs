use serde::{Deserialize, Serialize};

use super::{mysql::version::MySqlVersion, postgres::version::PostgreSQLVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Version {
    PostgreSQL(PostgreSQLVersion),
    MySql(MySqlVersion),
}

pub trait VersionTrait: Sized + ToString {
    fn from_str(string: &str) -> Option<Self>;
    fn parse_string_version(version_string: &str) -> Option<Self>;
}
