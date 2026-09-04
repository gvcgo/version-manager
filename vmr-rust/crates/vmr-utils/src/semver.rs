//! Version parsing and sorting (mirrors Go `vmr-go/internal/utils/sort_versions.go`).
//!
//! Ported-semantics notes (identical to Go, quirks included):
//! - Parsing: lowercase → take the first `digit(any-char-digit){0,2}` match → split by `.` into
//!   Major/Minor/Patch (Build is unreachable because the regex matches at most 3 segments; the
//!   field is kept in case the regex evolves). A version with no numeric segment (the `1.0`
//!   digit prefix is missing) counts as a parse failure.
//! - beta/rc: take the number from the first `beta\.*number` / `rc\.*number` segment of the whole
//!   string; when missing, set the max-integer sentinel (`i64::MAX`, mirroring Go `math.MaxInt`),
//!   which keeps **stable > rc > beta** among segments with equal numeric parts.
//! - Sorting: descending `sort_versions` / ascending `sort_versions_ascend`, comparing level by
//!   level: Major→Minor→Patch→Build→Beta→RC.
//! - Fallback: if either side fails to parse, comparison falls back to plain string comparison
//!   (Go compares whole rows while Rust has no row concept — for a list of single version strings
//!   the two are equivalent; for multi-column rows Go stringifies the whole row before comparing,
//!   whereas here the comparison is on the version string — see plan.md §3.2 "fallback to string
//!   comparison when parsing fails").
//! - Quirks preserved: the `.` in the version regex is unescaped (matches any character);
//!   `beta`/`rc` use naive substring detection (e.g. `3.2.0-source` contains the `rc` substring
//!   and is therefore treated as an rc version).

use std::cmp::Ordering;
use std::sync::LazyLock;

use regex::Regex;

/// Stable-version sentinel: the largest integer is used when beta/rc are missing
/// (Go `math.MaxInt`; on 64 bits that is `i64::MAX`).
const MAX_INT: i64 = i64::MAX;

/// Version segment: `\d+(.\d+){0,2}` (Go `versionRegexp`; the `.` is an unescaped quirk).
/// `\d` in Go's RE2 is ASCII-only, hence `[0-9]` is used.
static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+(.[0-9]+){0,2}").unwrap());
/// beta segment: `beta\.*\d+` (Go `betaRegexp`).
static BETA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"beta\.*[0-9]+").unwrap());
/// rc segment: `rc\.*\d+` (Go `rcRegexp`).
static RC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"rc\.*[0-9]+").unwrap());
/// In-segment number: `\d+` (Go `numRegexp`).
static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+").unwrap());

/// Numeric decomposition of a version string (mirrors Go `Version`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub build: i64,
    pub beta: i64,
    pub rc: i64,
}

/// Parse a version string; returns `None` when it contains no numeric segment (mirrors Go
/// returning an error).
pub fn parse_version(version: &str) -> Option<Version> {
    let version = version.to_lowercase();
    let numeric = VERSION_RE.find(&version)?.as_str();

    let mut v = Version {
        major: 0,
        minor: 0,
        patch: 0,
        build: 0,
        beta: 0,
        rc: 0,
    };
    for (idx, part) in numeric.split('.').enumerate() {
        // A numeric segment is always pure digits; on overflow/abnormality it falls back to 0
        // (mirrors Go gconv.Int).
        let n = part.parse::<i64>().unwrap_or(0);
        match idx {
            0 => v.major = n,
            1 => v.minor = n,
            2 => v.patch = n,
            3 => v.build = n,
            _ => break,
        }
    }

    // beta/rc number: take the first number string inside the first beta/rc segment of the whole string.
    v.beta = BETA_RE
        .find(&version)
        .and_then(|m| NUM_RE.find(m.as_str()))
        .map(|m| m.as_str().parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    v.rc = RC_RE
        .find(&version)
        .and_then(|m| NUM_RE.find(m.as_str()))
        .map(|m| m.as_str().parse::<i64>().unwrap_or(0))
        .unwrap_or(0);

    // Sentinel: when no beta/rc number exists — if the whole string contains that substring, treat
    // it as an unnumbered first release (1); otherwise it is a stable version (MAX_INT). The naive
    // substring-detection quirk is preserved.
    if v.beta == 0 {
        v.beta = if version.contains("beta") { 1 } else { MAX_INT };
    }
    if v.rc == 0 {
        v.rc = if version.contains("rc") { 1 } else { MAX_INT };
    }
    Some(v)
}

fn compare_fields_desc(a: &Version, b: &Version) -> Ordering {
    b.major
        .cmp(&a.major)
        .then_with(|| b.minor.cmp(&a.minor))
        .then_with(|| b.patch.cmp(&a.patch))
        .then_with(|| b.build.cmp(&a.build))
        .then_with(|| b.beta.cmp(&a.beta))
        .then_with(|| b.rc.cmp(&a.rc))
}

fn compare_fields_asc(a: &Version, b: &Version) -> Ordering {
    a.major
        .cmp(&b.major)
        .then_with(|| a.minor.cmp(&b.minor))
        .then_with(|| a.patch.cmp(&b.patch))
        .then_with(|| a.build.cmp(&b.build))
        .then_with(|| a.beta.cmp(&b.beta))
        .then_with(|| a.rc.cmp(&b.rc))
}

/// Compare two version strings in descending order: the larger version comes first.
/// If either side fails to parse, fall back to descending string comparison
/// (mirrors Go `SortVersions`).
fn compare_desc(a: &str, b: &str) -> Ordering {
    match (parse_version(a), parse_version(b)) {
        (Some(va), Some(vb)) => compare_fields_desc(&va, &vb),
        _ => b.cmp(a),
    }
}

/// Compare two version strings in ascending order: the smaller version comes first.
/// If either side fails to parse, fall back to ascending string comparison
/// (mirrors Go `SortVersionAscend`).
fn compare_asc(a: &str, b: &str) -> Ordering {
    match (parse_version(a), parse_version(b)) {
        (Some(va), Some(vb)) => compare_fields_asc(&va, &vb),
        _ => a.cmp(b),
    }
}

/// Sort a version list **descending** (in place, newest first).
///
/// Mirrors Go `utils.SortVersions` (the Go version sorts table rows by the leading version
/// string of each row; the Rust side operates directly on a list of version strings).
pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_desc(a, b));
}

/// Sort a version list **ascending** (in place, oldest first).
///
/// Mirrors Go `utils.SortVersionAscend`.
pub fn sort_versions_ascend(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_asc(a, b));
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX: i64 = MAX_INT;

    #[test]
    fn parse_plain_major_minor_patch() {
        let v = parse_version("1.2.3").unwrap();
        assert_eq!(
            v,
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
    }

    #[test]
    fn parse_partial_and_noisy_versions() {
        assert_eq!(
            parse_version("8").unwrap(),
            Version {
                major: 8,
                minor: 0,
                patch: 0,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
        assert_eq!(
            parse_version("1.2").unwrap(),
            Version {
                major: 1,
                minor: 2,
                patch: 0,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
        // Prefix/suffix noise is ignored; the first numeric segment is taken.
        assert_eq!(
            parse_version("go1.21.5").unwrap(),
            Version {
                major: 1,
                minor: 21,
                patch: 5,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
        assert_eq!(
            parse_version("v2.0.0-alpha").unwrap(),
            Version {
                major: 2,
                minor: 0,
                patch: 0,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
    }

    #[test]
    fn parse_beta_forms() {
        // beta1 / beta.1 / unnumbered beta / beta02: all beta segments; rc gets the sentinel.
        for (s, n) in [
            ("1.2.3-beta1", 1),
            ("1.2.3-beta.1", 1),
            ("1.2.3-beta", 1),
            ("1.2.3-beta02", 2),
        ] {
            let v = parse_version(s).unwrap();
            assert_eq!(v.beta, n, "{s} beta");
            assert_eq!(v.rc, MAX, "{s} rc 哨兵");
            assert_eq!((v.major, v.minor, v.patch), (1, 2, 3), "{s} 数值段");
        }
    }

    #[test]
    fn parse_rc_forms_case_insensitive() {
        for (s, n) in [("1.2.3-rc1", 1), ("1.2.3-rc.3", 3), ("1.2.3-RC2", 2)] {
            let v = parse_version(s).unwrap();
            assert_eq!(v.rc, n, "{s} rc");
            assert_eq!(v.beta, MAX, "{s} beta 哨兵");
            assert_eq!((v.major, v.minor, v.patch), (1, 2, 3), "{s} 数值段");
        }
    }

    #[test]
    fn parse_unparseable_returns_none() {
        for s in ["", "latest", "master", "no-digits-here"] {
            assert!(parse_version(s).is_none(), "{s:?} 应解析失败");
        }
    }

    #[test]
    fn build_field_unreachable_by_regex() {
        // The version regex matches at most 3 numeric segments; `.4` is not included → build is
        // always 0 (same in Go).
        assert_eq!(
            parse_version("1.2.3.4").unwrap(),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                build: 0,
                beta: MAX,
                rc: MAX
            }
        );
    }

    #[test]
    fn naive_beta_rc_substring_quirk() {
        // quirk: naive substring detection — "source" contains "rc" → treated as rc=1;
        // but the rc regex requires a digit after rc, which "3.2.0-source" lacks → rc takes the
        // 1 branch.
        let v = parse_version("3.2.0-source").unwrap();
        assert_eq!(v.beta, MAX);
        assert_eq!(v.rc, 1);
        // Hence it sorts after a stable version with the same numeric value.
        let mut list = vec!["3.2.0".to_string(), "3.2.0-source".to_string()];
        sort_versions(&mut list);
        assert_eq!(list, ["3.2.0", "3.2.0-source"]);
    }

    #[test]
    fn ordering_stable_over_rc_over_beta() {
        let stable = parse_version("1.2.3").unwrap();
        let rc = parse_version("1.2.3-rc1").unwrap();
        let beta = parse_version("1.2.3-beta1").unwrap();
        assert_eq!(compare_fields_desc(&stable, &rc), Ordering::Less); // stable first
        assert_eq!(compare_fields_desc(&rc, &beta), Ordering::Less); // rc first
    }

    #[test]
    fn sort_versions_descending() {
        let mut list = vec![
            "1.2.3-beta1".to_string(),
            "1.2.3".to_string(),
            "1.2.3-rc1".to_string(),
            "1.2.0".to_string(),
            "1.10.0".to_string(),
            "1.9.0".to_string(),
            "2.0.0".to_string(),
        ];
        sort_versions(&mut list);
        assert_eq!(
            list,
            [
                "2.0.0",
                "1.10.0",
                "1.9.0",
                "1.2.3",
                "1.2.3-rc1",
                "1.2.3-beta1",
                "1.2.0"
            ]
        );
    }

    #[test]
    fn sort_versions_ascending() {
        let mut list = vec![
            "2.0.0".to_string(),
            "1.2.3-rc1".to_string(),
            "1.2.3-beta1".to_string(),
            "1.9.0".to_string(),
            "1.2.3".to_string(),
        ];
        sort_versions_ascend(&mut list);
        assert_eq!(
            list,
            ["1.2.3-beta1", "1.2.3-rc1", "1.2.3", "1.9.0", "2.0.0"]
        );
    }

    #[test]
    fn sort_falls_back_to_lexical_on_unparseable() {
        // "latest" has no numeric segment → string comparison against both sides; a leading
        // letter > a leading digit.
        let mut list = vec![
            "latest".to_string(),
            "1.10.0".to_string(),
            "1.2.3".to_string(),
        ];
        sort_versions(&mut list);
        assert_eq!(list, ["latest", "1.10.0", "1.2.3"]);

        let mut list = vec![
            "latest".to_string(),
            "1.10.0".to_string(),
            "1.2.3".to_string(),
        ];
        sort_versions_ascend(&mut list);
        assert_eq!(list, ["1.2.3", "1.10.0", "latest"]);

        // Neither is parseable: plain string order.
        let mut list = vec!["beta".to_string(), "aaa".to_string()];
        sort_versions(&mut list);
        assert_eq!(list, ["beta", "aaa"]);
    }
}
