//! Validation for the identifiers that turn into directory names under `~/.vmr`.
//!
//! `sdk_name`, `plugin_name` and version names are concatenated into paths
//! (`<versions>/<sdk>_versions/<plugin>-<version>`, see `vmr-installer::common`). They arrive from
//! the command line, a project's `.vmr.lock`, MCP tool arguments, third-party plugins and remote
//! version metadata, so nothing but a strict allow-list may reach `Path::join`: a name such as
//! `../../projects/app` resolves outside the versions root, and `vmr uninstall` then calls
//! `remove_dir_all` on it.

/// Maximum accepted length of an identifier.
const MAX_LEN: usize = 128;

/// Each identifier component is displayed as-is in `vmr local` and in paths.
const PLACEHOLDER: &str = "_invalid";

/// Whether `value` is safe to use as a single path component.
///
/// Accepted: 1..=`MAX_LEN` bytes of ASCII alphanumerics and `.`, `_`, `+`, `-` (enough for
/// `node-v20.11.0`, `1.0.0-rc.1+build.5`, `2023-05-01`, `ripgrep`). Rejected: path separators, NUL,
/// whitespace, `.` and `..` themselves, and anything non-ASCII. The allow-list makes an escaping
/// component impossible, since `Path::join` can only treat `/` and `\` as separators.
pub fn is_valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LEN
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-'))
}

/// Validates an identifier, naming the field in the error.
pub fn check_name(kind: &str, value: &str) -> Result<(), String> {
    if is_valid_name(value) {
        return Ok(());
    }
    Err(format!(
        "invalid {kind} `{value}`: expected 1-{MAX_LEN} characters from [A-Za-z0-9._+-], \
         excluding `.` and `..`"
    ))
}

/// The name to use as a path component: `value` when valid, otherwise [`PLACEHOLDER`].
///
/// Callers validate their inputs with [`check_name`]; this keeps a missed call site inside the
/// versions root instead of letting it escape.
pub fn path_component(value: &str) -> &str {
    if is_valid_name(value) {
        value
    } else {
        PLACEHOLDER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_identifiers() {
        for ok in [
            "node",
            "nodejs",
            "ripgrep",
            "rustup",
            "go",
            "1.88.0",
            "v20.11.0",
            "1.0.0-rc.1+build.5",
            "2023-05-01",
            "Python3",
            "a_b",
        ] {
            assert!(is_valid_name(ok), "{ok} should be accepted");
            assert_eq!(path_component(ok), ok);
        }
    }

    #[test]
    fn rejects_traversal_and_separators() {
        for bad in [
            "",
            ".",
            "..",
            "../x",
            "../../projects/app",
            "a/b",
            "a\\b",
            "/etc/passwd",
            "C:\\x",
            "a\0b",
            "a b",
            "a:b",
            "sdk\n",
        ] {
            assert!(!is_valid_name(bad), "{bad:?} should be rejected");
            assert_eq!(path_component(bad), PLACEHOLDER);
        }
    }

    #[test]
    fn rejects_overlong_names() {
        assert!(is_valid_name(&"a".repeat(MAX_LEN)));
        assert!(!is_valid_name(&"a".repeat(MAX_LEN + 1)));
    }

    #[test]
    fn check_name_reports_the_field() {
        let err = check_name("sdk name", "../x").unwrap_err();
        assert!(err.contains("invalid sdk name"), "{err}");
        assert_eq!(check_name("version", "1.2.3"), Ok(()));
    }
}
