//! conda platform 映射（对齐 Go `lua_global/conda.go` 的 subdir 列表与
//! Os/Arch 解析，但改为纯函数供 Rust 全链路使用）。

/// conda subdir → 通用 arch（对齐 Go `ParseArch`）。
pub fn parse_arch(platform: &str) -> &'static str {
    match platform {
        "linux-64" | "win-64" | "osx-64" => "amd64",
        "linux-aarch64" | "win-arm64" | "osx-arm64" => "arm64",
        _ => "",
    }
}

/// conda subdir → vmr os 名（对齐 Go `ParseOS`）。
pub fn parse_os(platform: &str) -> &'static str {
    match platform {
        "linux-64" | "linux-aarch64" => "linux",
        "win-64" | "win-arm64" => "windows",
        "osx-64" | "osx-arm64" => "darwin",
        _ => "",
    }
}

/// 当前进程 os/arch → conda subdir；不支持返回 `None`。
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

/// 本机 conda subdir。
pub fn current_subdir() -> Option<&'static str> {
    let os = os_name();
    let arch = arch_name();
    platform_for(&os, &arch)
}

/// Rust os 名 → vmr os 名（Go `runtime.GOOS`）。
pub fn os_name() -> String {
    match std::env::consts::OS {
        "macos" => "darwin".to_string(),
        other => other.to_string(),
    }
}

/// Rust arch 名 → vmr arch 名（Go `runtime.GOARCH`）。
pub fn arch_name() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "amd64".to_string(),
        "aarch64" => "arm64".to_string(),
        "x86" => "386".to_string(),
        other => other.to_string(),
    }
}

/// os/arch 解析结果（供 Lua 侧与 CLI 复用）。
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
        // 当前平台一定能得到 subdir（本 CI 为 linux/amd64 或 aarch64）。
        let os = os_name();
        let arch = arch_name();
        assert!(matches!(os.as_str(), "linux" | "darwin" | "windows"));
        assert!(platform_for(&os, &arch).is_some());
    }
}
