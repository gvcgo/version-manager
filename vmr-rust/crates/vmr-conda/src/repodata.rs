//! repodata fetch/parse/cache + package query and install.
//!
//! repodata records: two tables, `packages` (.tar.bz2) and `packages.conda` (.conda),
//! keyed by package filename with the metadata as the value.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use vmr_core::conf::get_cache_retention_time;
use vmr_core::paths;
use vmr_net::fetcher::Fetcher;

use crate::platform::current_subdir;

/// default channel (conda-forge).
pub const DEFAULT_CHANNEL: &str = "https://conda.anaconda.org/conda-forge";

/// channel override env (custom source; default when empty).
const CHANNEL_ENV: &str = "VMR_CONDA_CHANNEL";

/// metadata for a single repodata package.
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

/// query result: one concrete package-file record (downloadable for install).
#[derive(Debug, Clone)]
pub struct RepoPackage {
    pub file: String,
    pub meta: RecordMeta,
    /// full download URL (channel/subdir/file).
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

/// fetch the current platform's repodata (or hit a fresh cache).
fn fetch_repodata() -> Result<Repodata, String> {
    let subdir = current_subdir().ok_or("unsupported platform for conda")?;
    let path = cache_file(subdir);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(data) = serde_json::from_str::<Repodata>(&content) {
            // mtime + retention time freshness check.
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
    // try the plain json first, then the .zst compressed variant.
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

/// all package versions (deduplicated, ascending string order, for display).
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

/// all records (with filename and URL), for install selection.
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

/// install selection: exact name+version; `.conda` container preferred (conda
/// ecosystem consensus), and within the same container type take the highest build_number.
pub fn select_package(name: &str, version: &str) -> Result<Option<RepoPackage>, String> {
    let all = query_packages(name)?;
    let mut matched: Vec<RepoPackage> = all
        .into_iter()
        .filter(|p| p.meta.version == version)
        .collect();
    matched.sort_by(|a, b| {
        let a_conda = a.file.ends_with(".conda");
        let b_conda = b.file.ends_with(".conda");
        // .conda first, then by build_number descending.
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

/// install a single package into prefix (plan D4 stage one: no recursive dependencies).
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
        // zip container: info-*.tar.zst metadata + pkg-*.tar.zst files.
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

/// unpack a tar stream into prefix (file entries; directories created implicitly by
/// parent MkdirAll, mirroring vmr-utils extract).
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
        // safety: reject absolute / `..` escapes.
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
