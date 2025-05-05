use serde::{Deserialize, Serialize};

use super::postgres::version::PostgreSQLVersionV2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Version {
    PostgreSQL(PostgreSQLVersionV2),
}

pub trait VersionTrait: Sized + ToString {
    fn from_str(string: &str) -> Option<Self>;
    fn parse_string_version(version_string: &str) -> Option<Self>;
}
