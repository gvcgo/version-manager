//! Cached downloads (mirrors Go `internal/download/sdk_file.go`).
//!
//! Cache path `<cache>/<plugin>/<version>/<file>`; idempotent (skipped when the file
//! already exists and passes checksum validation).
//! Downloads go through the vmr-net backend (mirror / reverse proxy / proxy chain,
//! multithreaded chunked download, checksum / size check).

use std::path::PathBuf;

use vmr_core::paths;
use vmr_lua::types::Item;
use vmr_net::{Fetcher, SumType, download_file};

fn sum_type_of(s: &str) -> Option<SumType> {
    match s.to_lowercase().as_str() {
        "sha1" => Some(SumType::Sha1),
        "sha256" => Some(SumType::Sha256),
        "sha512" => Some(SumType::Sha512),
        "md5" => Some(SumType::Md5),
        _ => None,
    }
}

/// Thread count (vmr-core policy; env is authoritative).
fn threads() -> usize {
    vmr_core::policy::get_download_thread_num().max(1) as usize
}

/// File name from the last URL segment (query stripped).
fn file_name_of(url: &str) -> String {
    let no_query = url.split(['?', '#']).next().unwrap_or(url);
    no_query
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "download".to_string())
}

/// Downloads to the cache and returns the path; returns None on failure or an empty URL.
pub fn download_to_cache(plugin_name: &str, version: &str, item: &Item) -> Option<PathBuf> {
    if item.url.is_empty() {
        return None;
    }
    let file = file_name_of(&item.url);
    let dest = paths::cache_dir()
        .join(plugin_name)
        .join(version)
        .join(&file);

    let checksum = if item.sum.is_empty() {
        None
    } else {
        sum_type_of(&item.sum_type).map(|st| vmr_net::Checksum {
            sum_type: st,
            value: item.sum.to_lowercase(),
        })
    };

    let client = Fetcher::for_url(&item.url).ok()?;
    let size = if item.size > 0 {
        Some(item.size as u64)
    } else {
        None
    };
    download_file(
        client.client(),
        &item.url,
        &dest,
        threads(),
        size,
        checksum,
        None,
    )
    .ok()?;
    Some(dest)
}

/// Removes the cached file (when present).
pub fn remove_cached(plugin_name: &str, version: &str, file: &str) {
    let p = paths::cache_dir()
        .join(plugin_name)
        .join(version)
        .join(file);
    let _ = std::fs::remove_file(p);
}

/// Whether the cached file is ready (present and passing checksum validation).
pub fn is_cached(plugin_name: &str, version: &str, item: &Item) -> bool {
    if item.url.is_empty() {
        return false;
    }
    let dest: PathBuf = paths::cache_dir()
        .join(plugin_name)
        .join(version)
        .join(file_name_of(&item.url));
    if !dest.exists() {
        return false;
    }
    if item.sum.is_empty() {
        return true;
    }
    let Some(st) = sum_type_of(&item.sum_type) else {
        return true;
    };
    vmr_net::checksum_of_file(&dest, st)
        .map(|h| h == item.sum.to_lowercase())
        .unwrap_or(false)
}
