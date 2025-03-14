use std::env;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    LinuxX64,
    LinuxAarch64,
    MacOsX64,
    MacOsAarch64,
    WindowsX64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Option<Self> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        match (os, arch) {
            ("linux", "x86_64") => Some(Platform::LinuxX64),
            ("linux", "aarch64") => Some(Platform::LinuxAarch64),
            ("macos", "x86_64") => Some(Platform::MacOsX64),
            ("macos", "aarch64") => Some(Platform::MacOsAarch64),
            ("windows", "x86_64") => Some(Platform::WindowsX64),
            _ => None,
        }
    }

    /// Convert platform to string identifier
    pub fn to_str(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux-x64",
            Platform::LinuxAarch64 => "linux-aarch64",
            Platform::MacOsX64 => "darwin-x64",
            Platform::MacOsAarch64 => "darwin-arm64",
            Platform::WindowsX64 => "windows-x64",
        }
    }

    /// Get the filename extension for binaries on this platform
    pub fn binary_ext(&self) -> &'static str {
        match self {
            Platform::WindowsX64 => ".exe",
            _ => "",
        }
    }

    /// Get the archive extension for this platform
    pub fn archive_ext(&self) -> &'static str {
        match self {
            Platform::WindowsX64 => ".zip",
            _ => ".tar.gz",
        }
    }
}
