use std::sync::LazyLock;

use regex::Regex;

static VERSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d+(.\d+){0,2}").unwrap());
static BETA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"beta\.*\d+").unwrap());
static RC_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"rc\.*\d+").unwrap());
static NUM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d+").unwrap());

/// Represents a parsed version number.
#[derive(Debug, Clone, Default)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub build: i32,
    pub beta: i32,
    pub rc: i32,
}

impl Version {
    /// Compare two versions for ordering.
    pub fn cmp_desc(&self, other: &Version) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| self.build.cmp(&other.build))
            .then_with(|| self.beta.cmp(&other.beta))
            .then_with(|| self.rc.cmp(&other.rc))
    }
}

/// Parse a semver-like string. Handles beta/rc suffixes.
/// Beta/RC default to `i32::MAX` if not present (so non-beta > beta).
/// If "beta" is in the string but no number, beta=1. Same for rc.
pub fn parse_version(version: &str) -> Result<Version, String> {
    let version_lower = version.to_lowercase();

    let vstr = VERSION_REGEX.find(&version_lower).map(|m| m.as_str().to_string());
    let bstr = BETA_REGEX.find(&version_lower).map(|m| m.as_str().to_string());
    let rstr = RC_REGEX.find(&version_lower).map(|m| m.as_str().to_string());

    let vstr = match vstr {
        Some(s) => s,
        None => return Err(format!("can not parse: {}", version)),
    };

    let parts: Vec<&str> = vstr.split('.').collect();
    let mut v = Version::default();

    for (i, part) in parts.iter().enumerate() {
        let num: i32 = part.parse().unwrap_or(0);
        match i {
            0 => v.major = num,
            1 => v.minor = num,
            2 => v.patch = num,
            3 => v.build = num,
            _ => {}
        }
    }

    // Parse beta
    let beta_num: i32 = bstr
        .as_ref()
        .and_then(|s| NUM_REGEX.find(s))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    if beta_num == 0 && !version_lower.contains("beta") {
        v.beta = i32::MAX;
    } else if beta_num == 0 && version_lower.contains("beta") {
        v.beta = 1;
    } else {
        v.beta = beta_num;
    }

    // Parse rc
    let rc_num: i32 = rstr
        .as_ref()
        .and_then(|s| NUM_REGEX.find(s))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    if rc_num == 0 && !version_lower.contains("rc") {
        v.rc = i32::MAX;
    } else if rc_num == 0 && version_lower.contains("rc") {
        v.rc = 1;
    } else {
        v.rc = rc_num;
    }

    Ok(v)
}

/// Sort version strings in descending order (newest first).
pub fn sort_versions_desc(versions: &mut [String]) {
    versions.sort_by(|a, b| {
        match (parse_version(a), parse_version(b)) {
            (Ok(v1), Ok(v2)) => v2.cmp_desc(&v1), // reverse for descending
            _ => b.cmp(a), // fallback to string comparison
        }
    });
}

/// Sort version strings in ascending order (oldest first).
pub fn sort_versions_asc(versions: &mut [String]) {
    versions.sort_by(|a, b| {
        match (parse_version(a), parse_version(b)) {
            (Ok(v1), Ok(v2)) => v1.cmp_desc(&v2),
            _ => a.cmp(b),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let v = parse_version("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = parse_version("1.2.3-beta.1").unwrap();
        assert_eq!(v.beta, 1);
        assert_eq!(v.rc, i32::MAX); // no rc → MAX

        let v = parse_version("1.2.3-rc.2").unwrap();
        assert_eq!(v.rc, 2);
        assert_eq!(v.beta, i32::MAX);

        // "beta" without number → beta = 1
        let v = parse_version("1.2.3-beta").unwrap();
        assert_eq!(v.beta, 1);
    }

    #[test]
    fn test_sort_desc() {
        let mut versions = vec![
            "1.0.0".to_string(),
            "2.0.0-beta.1".to_string(),
            "1.9.9".to_string(),
            "2.0.0".to_string(),
            "1.0.0-rc.1".to_string(),
        ];
        sort_versions_desc(&mut versions);
        assert!(versions[0].contains("2.0.0") && !versions[0].contains("beta"));
        assert!(versions[1].contains("beta"));
    }

    #[test]
    fn test_sort_asc() {
        let mut versions = vec![
            "2.0.0".to_string(),
            "1.0.0".to_string(),
            "1.5.0".to_string(),
        ];
        sort_versions_asc(&mut versions);
        assert_eq!(versions[0], "1.0.0");
        assert_eq!(versions[2], "2.0.0");
    }
}
