use core::fmt;
use std::str::FromStr;

use regex::Regex;

use crate::databases::DbVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostgreSQLVersion {
    V9_6,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
}

pub const DEFAULT_POSTGRES_VERSION: PostgreSQLVersion = PostgreSQLVersion::V15;

impl DbVersion for PostgreSQLVersion {
    fn as_str(&self) -> &'static str {
        match self {
            PostgreSQLVersion::V9_6 => "9.6",
            PostgreSQLVersion::V10 => "10",
            PostgreSQLVersion::V11 => "11",
            PostgreSQLVersion::V12 => "12",
            PostgreSQLVersion::V13 => "13",
            PostgreSQLVersion::V14 => "14",
            PostgreSQLVersion::V15 => "15",
            PostgreSQLVersion::V16 => "16",
            PostgreSQLVersion::V17 => "17",
        }
    }

    fn from_str(version: &str) -> Option<Self> {
        match version {
            "9.6" => Some(PostgreSQLVersion::V9_6),
            "10" => Some(PostgreSQLVersion::V10),
            "11" => Some(PostgreSQLVersion::V11),
            "12" => Some(PostgreSQLVersion::V12),
            "13" => Some(PostgreSQLVersion::V13),
            "14" => Some(PostgreSQLVersion::V14),
            "15" => Some(PostgreSQLVersion::V15),
            "16" => Some(PostgreSQLVersion::V16),
            "17" => Some(PostgreSQLVersion::V17),
            _ => None,
        }
    }

    fn from_version_tuple(major: u32, minor: u32, _patch: u32) -> Option<Self> {
        match (major, minor) {
            (9, 0) => Some(Self::V9_6),
            (10, 0) => Some(Self::V10),
            (11, 0) => Some(Self::V11),
            (12, 0) => Some(Self::V12),
            (13, 0) => Some(Self::V13),
            (14, 0) => Some(Self::V14),
            (15, 0) => Some(Self::V15),
            (16, 0) => Some(Self::V16),
            (17, 0) => Some(Self::V17),
            _ => None,
        }
    }

    fn parse_string_version(version_string: &str) -> Option<Self> {
        let pg_regex = Regex::new(r"PostgreSQL (\d+)\.(\d+)").ok()?;
        let captures = pg_regex.captures(version_string)?;

        let major = captures.get(1)?.as_str().parse::<u32>().ok()?;
        let minor = 0;
        let patch = 0;

        Self::from_version_tuple(major, minor, patch)
    }
}

impl fmt::Display for PostgreSQLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PostgreSQLVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        <PostgreSQLVersion as DbVersion>::from_str(s)
            .ok_or_else(|| format!("Unsupported PostgreSQL version: {}", s))
    }
}
