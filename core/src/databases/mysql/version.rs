use std::{fmt, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::databases::version::VersionTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl VersionTrait for MySqlVersion {
    fn from_str(string: &str) -> Option<Self> {
        let res: Vec<&str> = string.split(".").collect();

        let major = res.get(0)?.parse::<u16>().ok()?;
        let minor = res.get(1)?.parse::<u16>().ok()?;
        let patch = res.get(2)?.parse::<u16>().ok()?;

        Some(MySqlVersion {
            major,
            minor,
            patch,
        })
    }

    fn parse_string_version(version_string: &str) -> Option<Self> {
        let regex = Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok()?;
        let captures = regex.captures(version_string)?;

        let major = captures.get(1)?.as_str().parse::<u16>().ok()?;
        let minor = captures.get(2)?.as_str().parse::<u16>().ok()?;
        let patch = captures.get(3)?.as_str().parse::<u16>().ok()?;

        Some(MySqlVersion {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for MySqlVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.major)
    }
}

impl FromStr for MySqlVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        <MySqlVersion as VersionTrait>::from_str(s)
            .ok_or_else(|| format!("Unsupported PostgreSQL version: {}", s))
    }
}
