//! repodata 拉取/解析/缓存 + 包查询与安装。
//!
//! repodata 记录：`packages`（.tar.bz2）与 `packages.conda`（.conda）两张表，
//! 键为包文件名，值为元数据。

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use vmr_core::conf::get_cache_retention_time;
use vmr_core::paths;
use vmr_net::fetcher::Fetcher;

use crate::platform::current_subdir;

/// 默认 channel（conda-forge）。
pub const DEFAULT_CHANNEL: &str = "https://conda.anaconda.org/conda-forge";

/// channel 覆盖 env（自定义源；空则默认）。
const CHANNEL_ENV: &str = "VMR_CONDA_CHANNEL";

/// 单条 repodata 包元数据。
#[derive(Debug, Clone, Deserialize)]
pub struct RecordMeta {
    pub name: String,
    pub version: String,
    pub build: String,
    #[serde(default)]
    pub build_number: i64,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub size: i64,
}

/// 查询结果：一个具体包文件记录（可下载安装）。
#[derive(Debug, Clone)]
pub struct RepoPackage {
    pub file: String,
    pub meta: RecordMeta,
    /// 完整下载 URL（channel/subdir/file）。
    pub url: String,
}

pub fn channel() -> String {
    std::env::var(CHANNEL_ENV).unwrap_or_else(|_| DEFAULT_CHANNEL.to_string())
}

fn cache_file(subdir: &str) -> PathBuf {
    let dir = paths::cache_dir().join("conda_repodata");
    let name = format!("{}_{subdir}.json", sanitize(&channel()));
    dir.join(name)
}

fn sanitize(s: &str) -> String {
    s.replace(['/', ':', '.'], "_")
}

#[derive(Deserialize)]
struct Repodata {
    #[serde(default)]
    packages: BTreeMap<String, RecordMeta>,
    #[serde(default)]
    #[serde(rename = "packages.conda")]
    packages_conda: BTreeMap<String, RecordMeta>,
}

/// 拉取（或命中新鲜缓存）当前 platform 的 repodata。
fn fetch_repodata() -> Result<Repodata, String> {
    let subdir = current_subdir().ok_or("unsupported platform for conda")?;
    let path = cache_file(subdir);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(data) = serde_json::from_str::<Repodata>(&content) {
            // mtime + 保留时间判鲜。
            let fresh = fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .map(|d| d.as_secs() < get_cache_retention_time() as u64)
                .unwrap_or(false);
            if fresh {
                return Ok(data);
            }
        }
    }
    let raw = download_repodata_raw(subdir)?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, &raw);
    serde_json::from_slice(&raw).map_err(|e| format!("bad repodata: {e}"))
}

fn download_repodata_raw(subdir: &str) -> Result<Vec<u8>, String> {
    let base = channel();
    // 依次尝试普通 json 与 .zst 压缩版。
    let mut last_err = None;
    for u in [
        format!("{base}/{subdir}/repodata.json"),
        format!("{base}/{subdir}/repodata.json.zst"),
    ] {
        match raw_get_bytes(&u) {
            Ok(bytes) if u.ends_with(".zst") => {
                return zstd::stream::decode_all(bytes.as_slice())
                    .map_err(|e| format!("zstd decode failed: {e}"));
            }
            Ok(bytes) => return Ok(bytes),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| "repodata unavailable".to_string()))
}

fn raw_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let f = Fetcher::for_url(url).map_err(|e| e.to_string())?;
    f.get_bytes(url).map_err(|e| e.to_string())
}

/// 包全部版本（去重、字符串升序，展示用）。
pub fn query_versions(sdk_name: &str) -> Result<Vec<String>, String> {
    let data = fetch_repodata()?;
    let mut set = std::collections::BTreeSet::new();
    for rec in data.packages.values().chain(data.packages_conda.values()) {
        if rec.name == sdk_name {
            set.insert(rec.version.clone());
        }
    }
    Ok(set.into_iter().collect())
}

/// 全部记录（含文件名与 URL），供安装选择。
pub fn query_packages(sdk_name: &str) -> Result<Vec<RepoPackage>, String> {
    let data = fetch_repodata()?;
    let subdir = current_subdir().ok_or("unsupported platform")?;
    let base = channel();
    let mut out = Vec::new();
    for (file, rec) in data.packages.into_iter().chain(data.packages_conda) {
        if rec.name == sdk_name {
            let url = format!("{base}/{subdir}/{file}");
            out.push(RepoPackage {
                file,
                meta: rec,
                url,
            });
        }
    }
    Ok(out)
}

/// 安装选择：精确 name+version；`.conda` 容器优先（conda 生态共识），
/// 同容器类型取最高 build_number。
pub fn select_package(name: &str, version: &str) -> Result<Option<RepoPackage>, String> {
    let all = query_packages(name)?;
    let mut matched: Vec<RepoPackage> = all
        .into_iter()
        .filter(|p| p.meta.version == version)
        .collect();
    matched.sort_by(|a, b| {
        let a_conda = a.file.ends_with(".conda");
        let b_conda = b.file.ends_with(".conda");
        // .conda 优先，再按 build_number 降序。
        b_conda
            .cmp(&a_conda)
            .then(b.meta.build_number.cmp(&a.meta.build_number))
    });
    Ok(matched.into_iter().next())
}

fn temp_file() -> PathBuf {
    let dir = paths::temp_dir().join("conda");
    let _ = fs::create_dir_all(&dir);
    dir.join(format!(
        "pkg_{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

/// 安装单包到 prefix（plan D4 阶段一：不递归依赖）。
pub fn install_package(pkg: &RepoPackage, prefix: &Path) -> Result<(), String> {
    fs::create_dir_all(prefix).map_err(|e| e.to_string())?;
    let tmp = temp_file();
    let client = Fetcher::for_url(&pkg.url).map_err(|e| e.to_string())?;
    let checksum = if pkg.meta.sha256.is_empty() {
        None
    } else {
        Some(vmr_net::Checksum {
            sum_type: vmr_net::SumType::Sha256,
            value: pkg.meta.sha256.clone(),
        })
    };
    vmr_net::download_file(
        client.client(),
        &pkg.url,
        &tmp,
        1,
        Some(pkg.meta.size as u64),
        checksum,
        None,
    )
    .map_err(|e| format!("download {} failed: {e}", pkg.url))?;

    let result = extract_to_prefix(&tmp, prefix);
    let _ = fs::remove_file(&tmp);
    result
}

fn extract_to_prefix(pkg_path: &Path, prefix: &Path) -> Result<(), String> {
    if pkg_path.to_string_lossy().ends_with(".conda") {
        // zip 容器：info-*.tar.zst 元数据 + pkg-*.tar.zst 文件。
        let file = fs::File::open(pkg_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            if !name.starts_with("pkg-") || !name.ends_with(".tar.zst") {
                continue;
            }
            let dec = zstd::stream::read::Decoder::new(&mut entry).map_err(|e| e.to_string())?;
            return unpack_tar_stream(dec, prefix);
        }
        Err("no pkg-*.tar.zst inside .conda".to_string())
    } else if pkg_path.to_string_lossy().ends_with(".tar.bz2") {
        let file = fs::File::open(pkg_path).map_err(|e| e.to_string())?;
        let dec = bzip2::read::BzDecoder::new(file);
        unpack_tar_stream(dec, prefix)
    } else {
        Err(format!(
            "unsupported conda package format: {}",
            pkg_path.display()
        ))
    }
}

/// 解 tar 流到 prefix（文件项；目录由父级 MkdirAll 隐式创建，对齐 vmr-utils extract）。
fn unpack_tar_stream<R: Read>(reader: R, prefix: &Path) -> Result<(), String> {
    let mut ar = tar::Archive::new(reader);
    let entries = ar.entries().map_err(|e| e.to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let t = entry.header().entry_type();
        if !t.is_file() {
            continue;
        }
        let name = entry
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned();
        // 安全：拒绝绝对/`..` 逃逸。
        let p = std::path::Path::new(&name);
        if p.is_absolute() || name.split('/').any(|c| c == "..") {
            return Err(format!("unsafe path in package: {name}"));
        }
        let target = prefix.join(name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = fs::File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_meta_parses() {
        let json = r#"{"packages":{"lua-5.4.6-1.tar.bz2":{"name":"lua","version":"5.4.6","build":"1","build_number":1,"sha256":"abcd","size":123}},"packages.conda":{}}"#;
        let d: Repodata = serde_json::from_str(json).unwrap();
        assert_eq!(d.packages.len(), 1);
        let rec = d.packages.values().next().unwrap();
        assert_eq!(rec.name, "lua");
        assert_eq!(rec.version, "5.4.6");
        assert_eq!(rec.sha256, "abcd");
    }

    #[test]
    fn select_prefers_conda_container() {
        let mk = |file: &str, ver: &str, bn: i64| RepoPackage {
            file: file.to_string(),
            meta: RecordMeta {
                name: "x".into(),
                version: ver.into(),
                build: String::new(),
                build_number: bn,
                sha256: String::new(),
                size: 0,
            },
            url: String::new(),
        };
        let mut list = [
            mk("x-1.0-0.tar.bz2", "1.0", 2),
            mk("x-1.0-h1_0.conda", "1.0", 0),
            mk("x-1.0-h2_0.conda", "1.0", 5),
        ];
        list.sort_by(|a, b| {
            let a_c = a.file.ends_with(".conda");
            let b_c = b.file.ends_with(".conda");
            b_c.cmp(&a_c)
                .then(b.meta.build_number.cmp(&a.meta.build_number))
        });
        assert_eq!(list[0].file, "x-1.0-h2_0.conda");
    }
}
