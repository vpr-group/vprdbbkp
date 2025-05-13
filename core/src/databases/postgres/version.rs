use core::fmt;
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::databases::version::VersionTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgreSQLVersion {
    pub major: u16,
    pub minor: u16,
}

impl VersionTrait for PostgreSQLVersion {
    fn from_str(string: &str) -> Option<Self> {
        let res: Vec<&str> = string.split(".").collect();

        let major = res.get(0)?.parse::<u16>().ok()?;
        let minor = res.get(1)?.parse::<u16>().ok()?;

        Some(PostgreSQLVersion { major, minor })
    }

    fn parse_string_version(version_string: &str) -> Option<Self> {
        let pg_regex = Regex::new(r"PostgreSQL (\d+)\.(\d+)").ok()?;
        let captures = pg_regex.captures(version_string)?;

        let major = captures.get(1)?.as_str().parse::<u16>().ok()?;
        let minor = captures.get(2)?.as_str().parse::<u16>().ok()?;

        Some(PostgreSQLVersion { major, minor })
    }
}

impl fmt::Display for PostgreSQLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.major)
    }
}

impl FromStr for PostgreSQLVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        <PostgreSQLVersion as VersionTrait>::from_str(s)
            .ok_or_else(|| format!("Unsupported PostgreSQL version: {}", s))
    }
}
