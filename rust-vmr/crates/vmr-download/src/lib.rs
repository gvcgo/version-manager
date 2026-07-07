use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Version download info — mirrors Go's `lua_global.Item`.
#[derive(Debug, Clone)]
pub struct Item {
    pub url: String,
    pub arch: String,
    pub os: String,
    pub installer: String,
    pub sum: String,
    pub sum_type: String,
    pub size: i64,
}

/// SDK file downloader — mirrors Go's `request.Downloader`.
pub struct Downloader {
    pub sdk_name: String,
    pub version_name: String,
    pub version: Item,
}

impl Downloader {
    /// Create a new `Downloader` with empty defaults.
    pub fn new() -> Self {
        Self {
            sdk_name: String::new(),
            version_name: String::new(),
            version: Item {
                url: String::new(),
                arch: String::new(),
                os: String::new(),
                installer: String::new(),
                sum: String::new(),
                sum_type: String::new(),
                size: 0,
            },
        }
    }

    /// Download the SDK file. Returns the local file path on success,
    /// or an empty string when the URL is empty or the download is invalid.
    pub fn download(
        &mut self,
        origin_sdk_name: &str,
        version_name: &str,
        version: Item,
    ) -> String {
        self.sdk_name = origin_sdk_name.to_string();
        self.version_name = version_name.to_string();
        self.version = version;

        if self.version.url.is_empty() {
            return String::new();
        }

        let save_path = match self.build_local_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[vmr-download] failed to build local path: {e}");
                return String::new();
            }
        };

        // Skip if file already exists
        if save_path.exists() {
            return save_path.to_string_lossy().into_owned();
        }

        // Ensure parent directory exists
        if let Some(parent) = save_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("[vmr-download] failed to create dir {}: {e}", parent.display());
                return String::new();
            }
        }

        match self.do_download(&save_path) {
            Ok(()) => {
                // Discard tiny files — likely error pages
                if let Ok(meta) = save_path.metadata() {
                    if meta.len() <= 100 {
                        let _ = fs::remove_file(&save_path);
                        return String::new();
                    }
                }
                save_path.to_string_lossy().into_owned()
            }
            Err(e) => {
                eprintln!("[vmr-download] download failed: {e}");
                String::new()
            }
        }
    }

    /// Build the local file path: `{cache_dir}/{sdk_name}/{version_name}/{filename}`
    fn build_local_path(&self) -> io::Result<PathBuf> {
        let cache = vmr_config::paths::get_cache_dir();
        let file_name = extract_filename(&self.version.url, &self.sdk_name, &self.version_name);
        Ok(cache
            .join(&self.sdk_name)
            .join(&self.version_name)
            .join(file_name))
    }

    fn do_download(&self, save_path: &Path) -> Result<(), String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30 * 60))
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| format!("failed to create HTTP client: {e}"))?;

        let mut response = client
            .get(&self.version.url)
            .send()
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "HTTP {} for {}",
                response.status(),
                self.version.url
            ));
        }

        let mut file =
            fs::File::create(save_path).map_err(|e| format!("cannot create file: {e}"))?;
        let mut buf = [0u8; 8192];

        loop {
            let n = response
                .read(&mut buf)
                .map_err(|e| format!("read error: {e}"))?;
            if n == 0 {
                break;
            }
            io::Write::write_all(&mut file, &buf[..n])
                .map_err(|e| format!("write error: {e}"))?;
        }

        drop(file);

        // Verify checksum if sum & sum_type are set
        if !self.version.sum.is_empty() && !self.version.sum_type.is_empty() {
            verify_checksum(save_path, &self.version.sum, &self.version.sum_type)?;
        }

        Ok(())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the base filename from a URL.
///
/// Special case for Gradle: when the URL contains a query string (`?`),
/// the filename is forced to `gradle-{version}-all.zip`.
fn extract_filename(url: &str, sdk_name: &str, version_name: &str) -> String {
    // Strip query string and fragment
    let clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);

    // Special case: Gradle with query params
    if sdk_name.eq_ignore_ascii_case("gradle") && url.contains('?') {
        return format!("gradle-{}-all.zip", version_name);
    }

    let path = clean.trim_end_matches('/');
    path.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "download".to_string())
}

/// Verify a file's checksum against the expected hex value.
///
/// Currently supports `sha256`. Returns `Ok(())` on match, or an `Err` with a
/// human-readable message on mismatch / unsupported type.
pub fn verify_checksum(
    file_path: &Path,
    expected_sum: &str,
    sum_type: &str,
) -> Result<(), String> {
    // Gracefully skip if expected is empty
    if expected_sum.is_empty() {
        return Ok(());
    }

    match sum_type.to_lowercase().as_str() {
        "sha256" => {
            let mut file = fs::File::open(file_path)
                .map_err(|e| format!("cannot open file for checksum: {e}"))?;
            let mut hasher = Sha256::new();
            let mut buf = [0u8; 8192];

            loop {
                let n = file
                    .read(&mut buf)
                    .map_err(|e| format!("read error during checksum: {e}"))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }

            let hash = hex::encode(hasher.finalize());
            if hash.eq_ignore_ascii_case(expected_sum) {
                Ok(())
            } else {
                Err(format!(
                    "sha256 mismatch: expected {}, got {}",
                    expected_sum, hash
                ))
            }
        }
        other => Err(format!("unsupported checksum type: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_simple() {
        let name = extract_filename(
            "https://example.com/go1.21.0.linux-amd64.tar.gz",
            "golang",
            "1.21.0",
        );
        assert_eq!(name, "go1.21.0.linux-amd64.tar.gz");
    }

    #[test]
    fn test_extract_filename_gradle_with_query() {
        let name = extract_filename(
            "https://services.gradle.org/distributions/gradle-8.5-all.zip?foo=bar",
            "gradle",
            "8.5",
        );
        assert_eq!(name, "gradle-8.5-all.zip");
    }

    #[test]
    fn test_extract_filename_gradle_no_query() {
        let name = extract_filename(
            "https://services.gradle.org/distributions/gradle-8.5-bin.zip",
            "gradle",
            "8.5",
        );
        assert_eq!(name, "gradle-8.5-bin.zip");
    }

    #[test]
    fn test_verify_checksum_sha256_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"hello world").unwrap();

        // sha256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_checksum(&path, expected, "sha256").is_ok());
    }

    #[test]
    fn test_verify_checksum_sha256_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"hello world").unwrap();

        let result = verify_checksum(&path, "deadbeef", "sha256");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_checksum_unsupported_type() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"data").unwrap();

        let result = verify_checksum(&path, "abc", "md5");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported"));
    }

    #[test]
    fn test_verify_checksum_empty_sum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"data").unwrap();

        assert!(verify_checksum(&path, "", "sha256").is_ok());
    }
}
