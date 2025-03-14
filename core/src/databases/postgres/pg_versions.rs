#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostgresVersion {
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

impl PostgresVersion {
    // Get the version as a string (e.g., "9.6", "10", "15")
    pub fn as_str(&self) -> &'static str {
        match self {
            PostgresVersion::V9_6 => "9.6",
            PostgresVersion::V10 => "10",
            PostgresVersion::V11 => "11",
            PostgresVersion::V12 => "12",
            PostgresVersion::V13 => "13",
            PostgresVersion::V14 => "14",
            PostgresVersion::V15 => "15",
            PostgresVersion::V16 => "16",
            PostgresVersion::V17 => "17",
        }
    }

    // For package names (e.g., "postgresql-15")
    pub fn package_name(&self) -> String {
        format!("postgresql-{}", self.as_str())
    }

    // For contrib package
    pub fn contrib_package_name(&self) -> String {
        format!("postgresql-contrib-{}", self.as_str())
    }

    // For binary directory
    pub fn bin_path(&self) -> String {
        format!("/usr/lib/postgresql/{}/bin", self.as_str())
    }

    // Get all supported versions
    pub fn all() -> Vec<PostgresVersion> {
        vec![
            PostgresVersion::V9_6,
            PostgresVersion::V10,
            PostgresVersion::V11,
            PostgresVersion::V12,
            PostgresVersion::V13,
            PostgresVersion::V14,
            PostgresVersion::V15,
            PostgresVersion::V16,
            PostgresVersion::V17,
        ]
    }

    // Parse from string
    pub fn from_str(version: &str) -> Option<Self> {
        match version {
            "9.6" => Some(PostgresVersion::V9_6),
            "10" => Some(PostgresVersion::V10),
            "11" => Some(PostgresVersion::V11),
            "12" => Some(PostgresVersion::V12),
            "13" => Some(PostgresVersion::V13),
            "14" => Some(PostgresVersion::V14),
            "15" => Some(PostgresVersion::V15),
            "16" => Some(PostgresVersion::V16),
            "17" => Some(PostgresVersion::V17),
            _ => None,
        }
    }
}
