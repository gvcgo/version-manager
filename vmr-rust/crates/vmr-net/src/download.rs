//! Multithreaded chunked download + checksum/size verification (plan.md §3.3
//! requirement 8, modeled on goutils GetMultiPartFile).
//!
//! - HEAD fetches `Content-Length` → split into `Range`s by thread count → fetch the
//!   `.part{i}` temp files in parallel (in a `temp_part_xxx` directory) → merge them in
//!   order → remove the parts directory.
//! - Falls back to single-connection streaming download when the thread count is ≤1 or
//!   the server returns no length.
//! - Optional verification: after the download completes, verify the size and the
//!   sha1/sha256/sha512/md5 checksum.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::Client;
use sha2::Digest;

/// Checksum type (mirrors the values of `Item.SumType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumType {
    Sha1,
    Sha256,
    Sha512,
    Md5,
}

/// Expected checksum information.
#[derive(Debug, Clone)]
pub struct Checksum {
    pub sum_type: SumType,
    pub value: String,
}

/// Computes the file checksum (lowercase hex).
pub fn checksum_of_file(path: &Path, sum_type: SumType) -> io::Result<String> {
    let mut f = fs::File::open(path)?;
    match sum_type {
        SumType::Sha1 => digest_impl(&mut f, sha1::Sha1::new()),
        SumType::Sha256 => digest_impl(&mut f, sha2::Sha256::new()),
        SumType::Sha512 => digest_impl(&mut f, sha2::Sha512::new()),
        SumType::Md5 => digest_impl(&mut f, md5::Md5::new()),
    }
}

fn digest_impl<D: Digest>(reader: &mut dyn Read, mut d: D) -> io::Result<String> {
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        d.update(&buf[..n]);
    }
    let out = d.finalize();
    Ok(out.iter().map(|b| format!("{b:02x}")).collect())
}

/// Only non-json/toml large files (>1 MiB) are worth chunking; Go decides by file
/// extension + thread count.
const MIN_MULTIPART_SIZE: u64 = 1024 * 1024;

struct PartResult {
    index: usize,
    err: Option<String>,
}

/// Downloads `url` to `dest` (idempotent: skipped when the file already exists and
/// passes verification).
///
/// `threads`: chunk thread count; `expected_size`/`checksum` are optional; `timeout`:
/// per-chunk timeout.
pub fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    threads: usize,
    expected_size: Option<u64>,
    checksum: Option<Checksum>,
    timeout: Option<Duration>,
) -> io::Result<()> {
    if dest.exists() {
        if verify(dest, expected_size, checksum.as_ref()) {
            return Ok(()); // idempotent cache hit
        }
        let _ = fs::remove_file(dest); // verification failed → re-download
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // HEAD fetches the length (on failure, treat as not chunkable).
    let length = head_length(client, url, timeout);

    let use_multipart = threads > 1 && length.map(|l| l >= MIN_MULTIPART_SIZE).unwrap_or(false);

    if use_multipart {
        let len = length.unwrap();
        multipart_download(client, url, dest, len, threads, timeout)?;
    } else {
        single_download(client, url, dest, timeout)?;
    }

    if !verify(dest, expected_size, checksum.as_ref()) {
        return Err(io::Error::other(
            "downloaded file failed checksum/size verification",
        ));
    }
    Ok(())
}

fn verify(dest: &Path, size: Option<u64>, checksum: Option<&Checksum>) -> bool {
    let meta = match fs::metadata(dest) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if let Some(s) = size {
        if meta.len() != s {
            return false;
        }
    }
    if let Some(c) = checksum {
        if checksum_of_file(dest, c.sum_type)
            .map(|h| h != c.value)
            .unwrap_or(true)
        {
            return false;
        }
    }
    true
}

fn head_length(client: &Client, url: &str, timeout: Option<Duration>) -> Option<u64> {
    let mut req = client.head(url);
    if let Some(t) = timeout {
        req = req.timeout(t);
    }
    let resp = req.send().ok()?;
    resp.headers()
        .get(reqwest::header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
}

fn single_download(
    client: &Client,
    url: &str,
    dest: &Path,
    timeout: Option<Duration>,
) -> io::Result<()> {
    let mut req = client.get(url);
    if let Some(t) = timeout {
        req = req.timeout(t);
    }
    let mut resp = req.send().map_err(to_io)?;
    let mut out = fs::File::create(dest)?;
    resp.copy_to(&mut out).map_err(to_io)?;
    Ok(())
}

fn multipart_download(
    client: &Client,
    url: &str,
    dest: &Path,
    length: u64,
    threads: usize,
    timeout: Option<Duration>,
) -> io::Result<()> {
    let part_dir = part_dir_for(dest);
    fs::create_dir_all(&part_dir)?;
    let result = do_multipart(client, url, dest, &part_dir, length, threads, timeout);
    let _ = fs::remove_dir_all(&part_dir);
    result
}

fn part_dir_for(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());
    let parent = dest
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let uniq = format!(
        "temp_part_{}_{name}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    );
    parent.join(uniq)
}

fn do_multipart(
    client: &Client,
    url: &str,
    dest: &Path,
    part_dir: &Path,
    length: u64,
    threads: usize,
    timeout: Option<Duration>,
) -> io::Result<()> {
    let n = threads.min(length as usize).max(1);
    let part_len = length / n as u64;
    let name = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let base = client.clone();
    let url_owned = url.to_string();
    let part_dir_owned = part_dir.to_path_buf();

    let handles: Vec<_> = (0..n)
        .map(|i| {
            let base = base.clone();
            let url = url_owned.clone();
            let dir = part_dir_owned.clone();
            let name = name.clone();
            std::thread::spawn(move || -> PartResult {
                let start = i as u64 * part_len;
                let end = if i + 1 == n {
                    length - 1
                } else {
                    (i as u64 + 1) * part_len - 1
                };
                let part_path = dir.join(format!("{name}.part{i}"));
                let range = format!("bytes={start}-{end}");
                let mut req = base.get(&url).header(reqwest::header::RANGE, range);
                if let Some(t) = timeout {
                    req = req.timeout(t);
                }
                let resp = match req.send() {
                    Ok(r) => r,
                    Err(e) => {
                        return PartResult {
                            index: i,
                            err: Some(e.to_string()),
                        };
                    }
                };
                if !resp.status().is_success() {
                    return PartResult {
                        index: i,
                        err: Some(format!("range request failed: {}", resp.status())),
                    };
                }
                let mut out = match fs::File::create(&part_path) {
                    Ok(f) => f,
                    Err(e) => {
                        return PartResult {
                            index: i,
                            err: Some(e.to_string()),
                        };
                    }
                };
                let mut reader = resp;
                let mut buf = [0u8; 65536];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(k) => {
                            if out.write_all(&buf[..k]).is_err() {
                                return PartResult {
                                    index: i,
                                    err: Some("part write failed".to_string()),
                                };
                            }
                        }
                        Err(e) => {
                            return PartResult {
                                index: i,
                                err: Some(e.to_string()),
                            };
                        }
                    }
                }
                PartResult {
                    index: i,
                    err: None,
                }
            })
        })
        .collect();

    let mut results: Vec<PartResult> = Vec::with_capacity(n);
    for h in handles {
        results.push(
            h.join()
                .map_err(|_| io::Error::other("part thread panicked"))?,
        );
    }
    for r in &results {
        if let Some(e) = &r.err {
            return Err(io::Error::other(format!("part {} failed: {e}", r.index)));
        }
    }
    // Merge the parts in order.
    let mut out = fs::File::create(dest)?;
    for r in &results {
        let part_path = part_dir.join(format!("{name}.part{}", r.index));
        let mut f = fs::File::open(&part_path)?;
        io::copy(&mut f, &mut out)?;
    }
    Ok(())
}

fn to_io(e: reqwest::Error) -> io::Error {
    io::Error::other(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn checksum_hex(data: &[u8], sum_type: SumType) -> String {
        match sum_type {
            SumType::Sha1 => {
                let mut d = sha1::Sha1::new();
                d.update(data);
                format!("{:x}", d.finalize())
            }
            SumType::Sha256 => {
                let mut d = sha2::Sha256::new();
                d.update(data);
                format!("{:x}", d.finalize())
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn checksum_of_file_matches() {
        let tmp = std::env::temp_dir().join(format!("vmr-net-sum-{}", std::process::id()));
        fs::write(&tmp, b"hello checksum").unwrap();
        let h = checksum_of_file(&tmp, SumType::Sha256).unwrap();
        assert_eq!(h, checksum_hex(b"hello checksum", SumType::Sha256));
        let _ = fs::remove_file(&tmp);
    }
}
