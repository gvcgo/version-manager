//! Pixi-style package backend.
//!
//! The repodata fetching/caching code keeps VMR's small on-disk cache, while dependency solving
//! and prefix installation are delegated to the same rust crates pixi uses (`rattler_solve` and
//! `rattler::install::Installer`). This removes the need for a local conda/miniconda binary and
//! keeps link metadata, noarch handling, symlinks and prefix rewriting consistent with pixi.
//!
//! Repodata records live in two tables, `packages` (.tar.bz2) and `packages.conda` (.conda),
//! keyed by package filename.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;

use indicatif::{ProgressBar, ProgressStyle};
use rattler::install::{IndicatifReporter, Installer};
use rattler_cache::package_cache::PackageCache;
use rattler_conda_types::{
    MatchSpec, PackageRecord, ParseStrictness, RepoDataRecord, Subdir,
    package::DistArchiveIdentifier,
};
use rattler_solve::{SolverImpl, SolverTask};

use vmr_core::conf::get_cache_retention_time;
use vmr_core::paths;

use crate::platform::current_subdir;

/// default channel (conda-forge).
pub const DEFAULT_CHANNEL: &str = "https://conda.anaconda.org/conda-forge";

/// channel override env (custom source; default when empty). `VMR_PIXI_CHANNEL` is the new
/// name; `VMR_CONDA_CHANNEL` is still honored for existing setups.
const CHANNEL_ENV: &str = "VMR_PIXI_CHANNEL";
const LEGACY_CHANNEL_ENV: &str = "VMR_CONDA_CHANNEL";

/// Metadata needed for the textual `search` output.
#[derive(Debug, Clone, Default, Deserialize)]
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
    #[serde(default)]
    pub depends: Vec<String>,
}

pub fn channel() -> String {
    std::env::var(CHANNEL_ENV)
        .or_else(|_| std::env::var(LEGACY_CHANNEL_ENV))
        .unwrap_or_else(|_| DEFAULT_CHANNEL.to_string())
}

fn cache_file(subdir: &str) -> PathBuf {
    let dir = paths::cache_dir().join("pixi_repodata");
    // Decoded conda-forge repodata is ~400 MB; keep the cache zstd-compressed (~60 MB).
    let name = format!("{}_{subdir}.json.zst", sanitize(&channel()));
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

/// Full ratler records for a subdirectory, used by the solver/installer.
#[derive(Deserialize)]
struct RawRepodata {
    #[serde(default)]
    packages: BTreeMap<String, PackageRecord>,
    #[serde(default)]
    #[serde(rename = "packages.conda")]
    packages_conda: BTreeMap<String, PackageRecord>,
}

/// Reads a fresh cached repodata JSON blob (when `use_cache`), or downloads and caches it.
fn cached_repodata_json(subdir: &str, use_cache: bool) -> Result<Vec<u8>, String> {
    let path = cache_file(subdir);
    let fresh = fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs() < get_cache_retention_time() as u64)
        .unwrap_or(false);
    if use_cache && fresh {
        if let Ok(compressed) = fs::read(&path) {
            if let Ok(raw) = zstd::stream::decode_all(compressed.as_slice()) {
                return Ok(raw);
            }
        }
    }

    let (cache_bytes, raw) = download_repodata(subdir)?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, &cache_bytes);
    Ok(raw)
}

/// Parses a repodata JSON blob while showing the same dot spinner used by other network work.
///
/// The download itself already has a byte progress bar; this spinner covers the (sometimes
/// several-second) decompression/parsing phase, which otherwise looks like VMR is hung.
fn parse_repodata<T: DeserializeOwned>(raw: &[u8], label: &str) -> Result<T, String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}").expect("valid progress template"),
    );
    pb.set_message(format!("searching pixi repodata: {label}"));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let result = serde_json::from_slice(raw).map_err(|e| format!("bad repodata for {label}: {e}"));
    pb.finish_and_clear();
    result
}

/// Fetch one subdir's repodata.
fn fetch_repodata_for(subdir: &str, use_cache: bool) -> Result<Repodata, String> {
    let raw = cached_repodata_json(subdir, use_cache)?;
    parse_repodata(&raw, subdir)
}

/// Fetch the current platform's repodata.
fn fetch_repodata(use_cache: bool) -> Result<Repodata, String> {
    let subdir = current_subdir().ok_or("unsupported platform for pixi")?;
    fetch_repodata_for(subdir, use_cache)
}

/// Returns `(bytes to cache, decoded JSON)`.
fn download_repodata(subdir: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let base = channel();
    // Prefer the .zst variant: conda-forge's plain repodata.json is several hundred MB.
    let mut last_err = None;
    for u in [
        format!("{base}/{subdir}/repodata.json.zst"),
        format!("{base}/{subdir}/repodata.json"),
    ] {
        match raw_get_bytes(&u) {
            Ok(bytes) if u.ends_with(".zst") => {
                let raw = zstd::stream::decode_all(bytes.as_slice())
                    .map_err(|e| format!("zstd decode failed: {e}"))?;
                return Ok((bytes, raw));
            }
            Ok(bytes) => {
                let compressed = zstd::stream::encode_all(bytes.as_slice(), 3)
                    .map_err(|e| format!("zstd encode failed: {e}"))?;
                return Ok((compressed, bytes));
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| "repodata unavailable".to_string()))
}

fn raw_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    // `Fetcher::get_bytes` does not verify Content-Length; a proxy can truncate a large
    // repodata response. Use the shared downloader (HEAD size check + Range retry).
    let tmp = paths::temp_dir().join(format!(
        "repodata_{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let result = (|| -> Result<Vec<u8>, String> {
        vmr_net::download_url(
            url,
            &tmp,
            4,
            None,
            None,
            Some(std::time::Duration::from_secs(3600)),
        )
        .map_err(|e| e.to_string())?;
        fs::read(&tmp).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&tmp);
    result
}

/// All package versions (newest first, deduplicated, for display); cached repodata is used
/// when fresh.
pub fn query_versions(sdk_name: &str) -> Result<Vec<String>, String> {
    query_versions_cached(sdk_name, true)
}

/// `query_versions` with an explicit cache toggle: `use_cache = false` always (re)downloads
/// the repodata, ignoring `CacheRetentionTime`.
pub fn query_versions_cached(sdk_name: &str, use_cache: bool) -> Result<Vec<String>, String> {
    let mut set = std::collections::BTreeSet::new();
    // Platform packages plus noarch packages (some tools are published only as noarch).
    for data in [
        fetch_repodata(use_cache)?,
        fetch_repodata_for("noarch", use_cache)?,
    ] {
        for rec in data.packages.values().chain(data.packages_conda.values()) {
            if rec.name == sdk_name {
                set.insert(rec.version.clone());
            }
        }
    }
    let mut versions: Vec<String> = set.into_iter().collect();
    vmr_utils::semver::sort_versions(&mut versions);
    Ok(versions)
}

/// Converts one repodata table to `RepoDataRecord`s for the given channel/subdir.
fn records_of(
    packages: BTreeMap<String, PackageRecord>,
    base: &str,
    subdir: &str,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<RepoDataRecord>,
) -> Result<(), String> {
    for (file, package_record) in packages {
        // The same filename can appear in both a platform subdir and `noarch`; the solver
        // rejects duplicate records, so the platform-specific entry (loaded first) wins.
        if !seen.insert(file.clone()) {
            continue;
        }
        let identifier = file
            .parse::<DistArchiveIdentifier>()
            .map_err(|e| format!("invalid package filename {file}: {e}"))?;
        let url = url::Url::parse(&format!("{base}/{subdir}/{file}"))
            .map_err(|e| format!("invalid package URL for {file}: {e}"))?;
        out.push(RepoDataRecord {
            package_record,
            identifier,
            url,
            channel: Some(base.to_string()),
        });
    }
    Ok(())
}

/// Loads available records for the current platform plus the `noarch` subdir.
fn available_records() -> Result<Vec<RepoDataRecord>, String> {
    let subdir = current_subdir().ok_or("unsupported platform for pixi")?;
    let base = channel();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let raw: RawRepodata = parse_repodata(&cached_repodata_json(subdir, true)?, subdir)?;
    records_of(raw.packages, &base, subdir, &mut seen, &mut out)?;
    records_of(raw.packages_conda, &base, subdir, &mut seen, &mut out)?;

    let raw: RawRepodata = parse_repodata(&cached_repodata_json("noarch", true)?, "noarch")?;
    records_of(raw.packages, &base, "noarch", &mut seen, &mut out)?;
    records_of(raw.packages_conda, &base, "noarch", &mut seen, &mut out)?;

    Ok(out)
}

/// Resolves a package and its dependency closure using the same resolver pixi uses.
pub fn resolve_packages(name: &str, version: &str) -> Result<Vec<RepoDataRecord>, String> {
    let records = available_records()?;
    let spec = MatchSpec::from_str(&format!("{name}=={version}"), ParseStrictness::Lenient)
        .map_err(|e| format!("invalid package spec {name}=={version}: {e}"))?;

    let virtual_packages = rattler_virtual_packages::VirtualPackage::detect(
        &rattler_virtual_packages::VirtualPackageOverrides::default(),
        None,
    )
    .map_err(|e| format!("failed to detect virtual packages: {e}"))?
    .into_iter()
    .map(Into::into)
    .collect();

    let mut task = SolverTask::from_iter([records.as_slice()]);
    task.specs = vec![spec];
    task.virtual_packages = virtual_packages;

    let solved = rattler_solve::resolvo::Solver
        .solve(task)
        .map_err(|e| format!("pixi solver failed: {e}"))?;
    Ok(solved.records)
}

/// Installs a solved record set into `prefix` using rattler's prefix installer.
///
/// Adapted from pixi's `crates/pixi_command_dispatcher/src/install_binary.rs`
/// (`install_binary_records`): target platform, package cache, download client and link-script
/// policy are wired the same way; only pixi's compute-engine/reporting layers are omitted.
pub fn install_packages(records: &[RepoDataRecord], prefix: &Path) -> Result<(), String> {
    fs::create_dir_all(prefix).map_err(|e| e.to_string())?;

    let target = Subdir::from_str(current_subdir().unwrap_or("noarch"))
        .or_else(|_| Subdir::from_str("noarch"))
        .map_err(|e| e.to_string())?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create async runtime: {e}"))?;

    let package_cache = PackageCache::new(paths::cache_dir().join("pixi_pkgs"));
    let mut http_builder =
        reqwest::Client::builder().user_agent(format!("vmr/{}", env!("CARGO_PKG_VERSION")));
    if let Ok(proxy) = std::env::var(vmr_core::envs::LOCAL_PROXY) {
        if !proxy.is_empty() {
            http_builder =
                http_builder.proxy(reqwest::Proxy::all(&proxy).map_err(|e| e.to_string())?);
        }
    }
    let http = http_builder
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;
    let download_client = reqwest_middleware::ClientBuilder::new(http).build();

    let installer = Installer::new()
        .with_target_platform(target)
        .with_package_cache(package_cache)
        .with_download_client(download_client)
        .with_execute_link_scripts(false)
        // Same progress reporting mechanism pixi uses for prefix installs. It reports
        // download, extraction, validation and linking progress on stderr.
        .with_reporter(IndicatifReporter::builder().finish());

    runtime
        .block_on(installer.install(prefix, records.to_vec()))
        .map(|_| ())
        .map_err(|e| format!("pixi install failed: {e}"))
}

use std::str::FromStr;

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
    fn package_record_deserializes_from_repodata() {
        // Minimal record shape used by the solver. Ensures rattler's PackageRecord accepts the
        // repodata JSON directly (the solver/installer path depends on this).
        let json = r#"{"name":"lua","version":"5.4.6","build":"h1_0","build_number":0,"depends":["libgcc-ng >=12"],"size":123,"sha256":"0000000000000000000000000000000000000000000000000000000000000000","subdir":"linux-64","md5":"00000000000000000000000000000000"}"#;
        let rec: PackageRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.name.as_normalized(), "lua");
        assert_eq!(rec.depends, vec!["libgcc-ng >=12"]);
    }
}
