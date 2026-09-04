//! 版本解析与排序（对齐 Go `vmr-go/internal/utils/sort_versions.go`）。
//!
//! 移植语义要点（与 Go 完全一致，含 quirk）：
//! - 解析：小写化 → 取首个 `数字(任意字符+数字){0,2}` 匹配段 → 按 `.` 拆出
//!   Major/Minor/Patch（Build 因正则最多 3 段而不可达，保留字段以备正则演进）。
//!   版本段中无数字（`1.0` 前缀缺失）视为解析失败。
//! - beta/rc：从整串取首个 `beta\.*数字` / `rc\.*数字` 段中的数字；
//!   缺失时置最大整数哨兵（`i64::MAX`，对齐 Go `math.MaxInt`），
//!   保证数值相同段间 **稳定版 > rc > beta**。
//! - 排序：降序 `sort_versions` / 升序 `sort_versions_ascend`，逐级比较
//!   Major→Minor→Patch→Build→Beta→RC。
//! - 回退：任一侧解析失败即退化为纯字符串比较（Go 比较整行、Rust 无行概念，
//!   对单版本字符串列表二者等价；多列行场景 Go 会把整行字符串化后比较，
//!   此处按版本字符串比较——见 plan.md §3.2「解析失败回退字符串比较」）。
//! - quirk 保留：版本正则 `.` 未转义（匹配任意字符）；`beta`/`rc` 为朴素
//!   子串检测（如 `3.2.0-source` 因含 `rc` 子串被当作 rc 版）。

use std::cmp::Ordering;
use std::sync::LazyLock;

use regex::Regex;

/// 稳定版哨兵：beta/rc 缺失时置最大整数（Go `math.MaxInt`，64 位 = `i64::MAX`）。
const MAX_INT: i64 = i64::MAX;

/// 版本号段：`\d+(.\d+){0,2}`（Go `versionRegexp`，`.` 未转义 quirk）。
/// Go RE2 的 `\d` 仅 ASCII，故用 `[0-9]`。
static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+(.[0-9]+){0,2}").unwrap());
/// beta 段：`beta\.*\d+`（Go `betaRegexp`）。
static BETA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"beta\.*[0-9]+").unwrap());
/// rc 段：`rc\.*\d+`（Go `rcRegexp`）。
static RC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"rc\.*[0-9]+").unwrap());
/// 段内数字：`\d+`（Go `numRegexp`）。
static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+").unwrap());

/// 一个版本号的数值分解（对齐 Go `Version`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub build: i64,
    pub beta: i64,
    pub rc: i64,
}

/// 解析版本字符串；不含任何数字段时返回 `None`（对齐 Go 返回 error）。
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
        // 数字段一定是纯数字；超长溢出/异常时归 0（对齐 Go gconv.Int）。
        let n = part.parse::<i64>().unwrap_or(0);
        match idx {
            0 => v.major = n,
            1 => v.minor = n,
            2 => v.patch = n,
            3 => v.build = n,
            _ => break,
        }
    }

    // beta/rc 数字：取整串中首个 beta/rc 段内的首个数字串。
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

    // 哨兵：无 beta/rc 数字时——整串含该子串则视为无编号首版（1），
    // 否则为稳定版（MAX_INT）。朴素子串检测 quirk 保留。
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

/// 降序比较两个版本字符串：大版本在前。
/// 任一侧解析失败回退为字符串降序比较（对齐 Go `SortVersions`）。
fn compare_desc(a: &str, b: &str) -> Ordering {
    match (parse_version(a), parse_version(b)) {
        (Some(va), Some(vb)) => compare_fields_desc(&va, &vb),
        _ => b.cmp(a),
    }
}

/// 升序比较两个版本字符串：小版本在前。
/// 任一侧解析失败回退为字符串升序比较（对齐 Go `SortVersionAscend`）。
fn compare_asc(a: &str, b: &str) -> Ordering {
    match (parse_version(a), parse_version(b)) {
        (Some(va), Some(vb)) => compare_fields_asc(&va, &vb),
        _ => a.cmp(b),
    }
}

/// 版本列表**降序**排序（原地，最新在前）。
///
/// 对齐 Go `utils.SortVersions`（Go 版作用于表格行、按行首版本字符串排序；
/// Rust 侧直接作用于版本字符串列表）。
pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_desc(a, b));
}

/// 版本列表**升序**排序（原地，最旧在前）。
///
/// 对齐 Go `utils.SortVersionAscend`。
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
        // 前缀/后缀噪声被忽略；取首个数字段。
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
        // beta1 / beta.1 / beta 无编号 / beta02：均为 beta 段，rc 置哨兵。
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
        // 版本正则最多 3 个数字段，`.4` 不被纳入 → build 恒 0（Go 同）。
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
        // quirk：子串检测朴素——"source" 含 "rc" → 被当作 rc=1 版；
        // 但 rc 正则需 rc 后跟数字，"3.2.0-source" 中无匹配 → rc 走 1 分支。
        let v = parse_version("3.2.0-source").unwrap();
        assert_eq!(v.beta, MAX);
        assert_eq!(v.rc, 1);
        // 因此它排在同数值稳定版之后。
        let mut list = vec!["3.2.0".to_string(), "3.2.0-source".to_string()];
        sort_versions(&mut list);
        assert_eq!(list, ["3.2.0", "3.2.0-source"]);
    }

    #[test]
    fn ordering_stable_over_rc_over_beta() {
        let stable = parse_version("1.2.3").unwrap();
        let rc = parse_version("1.2.3-rc1").unwrap();
        let beta = parse_version("1.2.3-beta1").unwrap();
        assert_eq!(compare_fields_desc(&stable, &rc), Ordering::Less); // stable 在前
        assert_eq!(compare_fields_desc(&rc, &beta), Ordering::Less); // rc 在前
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
        // "latest" 无数字段 → 与两侧均走字符串比较；字母开头 > 数字开头。
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

        // 两个都不可解析：纯字符串序。
        let mut list = vec!["beta".to_string(), "aaa".to_string()];
        sort_versions(&mut list);
        assert_eq!(list, ["beta", "aaa"]);
    }
}
