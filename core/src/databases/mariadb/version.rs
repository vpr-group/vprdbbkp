use core::fmt;
use std::str::FromStr;

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MariaDBVersion {
    V10_5,
    V10_6,
    V10_11,
    V11_0,
    V11_1,
    V11_2,
    V11_3,
    V11_4,
    V11_5,
    V11_6,
    V11_7,
}

pub const DEFAULT_MARIADB_VERSION: MariaDBVersion = MariaDBVersion::V11_2;

impl MariaDBVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V11_0 => "11.0",
            Self::V11_1 => "11.1",
            Self::V11_2 => "11.2",
            Self::V11_3 => "11.3",
            Self::V11_4 => "11.4",
            Self::V11_5 => "11.5",
            Self::V11_6 => "11.6",
            Self::V11_7 => "11.7",
            Self::V10_5 => "10.5",
            Self::V10_6 => "10.6",
            Self::V10_11 => "10.11",
        }
    }

    pub fn from_str(version: &str) -> Option<Self> {
        if version.starts_with("10.5") {
            Some(Self::V10_5)
        } else if version.starts_with("10.6") {
            Some(Self::V10_6)
        } else if version.starts_with("10.11") {
            Some(Self::V10_11)
        } else if version.starts_with("11.0") {
            Some(Self::V11_0)
        } else if version.starts_with("11.1") {
            Some(Self::V11_1)
        } else if version.starts_with("11.2") {
            Some(Self::V11_2)
        } else if version.starts_with("11.3") {
            Some(Self::V11_3)
        } else if version.starts_with("11.4") {
            Some(Self::V11_4)
        } else if version.starts_with("11.5") {
            Some(Self::V11_5)
        } else if version.starts_with("11.6") {
            Some(Self::V11_6)
        } else if version.starts_with("11.7") {
            Some(Self::V11_7)
        } else {
            None
        }
    }

    pub fn from_version_tuple(major: u32, minor: u32, _patch: u32) -> Option<Self> {
        match (major, minor) {
            (10, 5) => Some(Self::V10_5),
            (10, 6) => Some(Self::V10_6),
            (10, 11) => Some(Self::V10_11),
            (11, 0) => Some(Self::V11_0),
            (11, 1) => Some(Self::V11_1),
            (11, 2) => Some(Self::V11_2),
            (11, 3) => Some(Self::V11_3),
            (11, 4) => Some(Self::V11_4),
            (11, 5) => Some(Self::V11_5),
            (11, 6) => Some(Self::V11_6),
            (11, 7) => Some(Self::V11_7),
            _ => None,
        }
    }

    pub fn parse_string_version(version_string: &str) -> Option<MariaDBVersion> {
        let re = Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok()?;
        let captures = re.captures(version_string)?;

        let major = captures.get(1)?.as_str().parse::<u32>().ok()?;
        let minor = captures.get(2)?.as_str().parse::<u32>().ok()?;
        let patch = captures.get(3)?.as_str().parse::<u32>().ok()?;

        MariaDBVersion::from_version_tuple(major, minor, patch)
    }
}

impl fmt::Display for MariaDBVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MariaDBVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Unsupported MariaDB version: {}", s))
    }
}
