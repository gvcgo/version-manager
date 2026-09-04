//! conda platform mapping (mirrors Go `lua_global/conda.go`'s subdir list and
//! Os/Arch parsing, but turned into pure functions for the whole Rust pipeline).

/// conda subdir → generic arch (mirrors Go `ParseArch`).
pub fn parse_arch(platform: &str) -> &'static str {
    match platform {
        "linux-64" | "win-64" | "osx-64" => "amd64",
        "linux-aarch64" | "win-arm64" | "osx-arm64" => "arm64",
        _ => "",
    }
}

/// conda subdir → vmr os name (mirrors Go `ParseOS`).
pub fn parse_os(platform: &str) -> &'static str {
    match platform {
        "linux-64" | "linux-aarch64" => "linux",
        "win-64" | "win-arm64" => "windows",
        "osx-64" | "osx-arm64" => "darwin",
        _ => "",
    }
}

/// current process os/arch → conda subdir; returns `None` if unsupported.
pub fn platform_for(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "amd64") => Some("linux-64"),
        ("linux", "arm64") => Some("linux-aarch64"),
        ("darwin", "amd64") => Some("osx-64"),
        ("darwin", "arm64") => Some("osx-arm64"),
        ("windows", "amd64") => Some("win-64"),
        ("windows", "arm64") => Some("win-arm64"),
        _ => None,
    }
}

/// the local machine's conda subdir.
pub fn current_subdir() -> Option<&'static str> {
    let os = os_name();
    let arch = arch_name();
    platform_for(&os, &arch)
}

/// Rust os name → vmr os name (Go `runtime.GOOS`).
pub fn os_name() -> String {
    match std::env::consts::OS {
        "macos" => "darwin".to_string(),
        other => other.to_string(),
    }
}

/// Rust arch name → vmr arch name (Go `runtime.GOARCH`).
pub fn arch_name() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "amd64".to_string(),
        "aarch64" => "arm64".to_string(),
        "x86" => "386".to_string(),
        other => other.to_string(),
    }
}

/// os/arch parsing result (reused by the Lua side and the CLI).
pub struct PlatformParse {
    pub os: String,
    pub arch: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_roundtrip() {
        assert_eq!(platform_for("linux", "amd64"), Some("linux-64"));
        assert_eq!(platform_for("darwin", "arm64"), Some("osx-arm64"));
        assert_eq!(platform_for("windows", "arm64"), Some("win-arm64"));
        assert_eq!(platform_for("linux", "arm64"), Some("linux-aarch64"));
        assert_eq!(platform_for("freebsd", "amd64"), None);
        assert_eq!(parse_os("osx-64"), "darwin");
        assert_eq!(parse_arch("linux-aarch64"), "arm64");
    }

    #[test]
    fn host_names_normalized() {
        // the current platform must always yield a subdir (this CI is linux/amd64 or aarch64).
        let os = os_name();
        let arch = arch_name();
        assert!(matches!(os.as_str(), "linux" | "darwin" | "windows"));
        assert!(platform_for(&os, &arch).is_some());
    }
}
