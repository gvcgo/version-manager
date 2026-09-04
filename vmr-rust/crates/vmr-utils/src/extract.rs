//! Compression/archive extraction (corresponds to plan.md §3.2 `extract.rs`).
//!
//! Mirrors the two Go entry points:
//! - `unarchive`: corresponds to `Extractor.Unarchive` in
//!   `internal/utils/extract/extractor.go`, the concrete implementation behind Lua
//!   `vmrUnarchive` — the format is identified by **magic number**; single-stream compression
//!   (gz/bz2/xz/zst) is first decoded into `<dest>/temp/` and re-sniffed: archives are then
//!   extracted into dest; otherwise the result is treated as a "compressed single file", copied
//!   or renamed into dest, and given the `0111` executable bits when `single_exe` is set.
//! - `extract`: corresponds to `Extract` in `internal/utils/extractor.go` — **system commands are
//!   preferred** (`.zip`→`unzip -d`, `.tar*`→`tar -xf -C`, mirroring Go); on success, nested
//!   `.zip` files at the top level are expanded (`handleMultiCompress`); when the system commands
//!   are unavailable or fail, it falls back to the library implementation.
//!
//! Behavior mirrors Go's quirk: **directory entries inside an archive are not created on their
//! own** (parents are created implicitly via MkdirAll from file entries → empty directories are
//! dropped). Documented differences from Go (security hardening): entries with `..` or absolute
//! paths are rejected (zip-slip guard) — Go's plain Join risks path escape; archive symlink
//! entries are skipped.

use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::exec::exec;

/// Archive/compression format (magic-number identification; equivalent to mholt/archives Identify).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Gzip,
    Bzip2,
    Xz,
    Zstd,
    Zip,
    Tar,
    Unknown,
}

fn kind_of(bytes: &[u8]) -> Kind {
    if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        return Kind::Gzip;
    }
    if bytes.len() >= 3 && &bytes[0..3] == b"BZh" {
        return Kind::Bzip2;
    }
    if bytes.len() >= 6 && bytes[..6] == [0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00] {
        return Kind::Xz;
    }
    if bytes.len() >= 4 && bytes[..4] == [0x28, 0xb5, 0x2f, 0xfd] {
        return Kind::Zstd;
    }
    if bytes.len() >= 4 && bytes[..4] == *b"PK\x03\x04" {
        return Kind::Zip;
    }
    // An empty zip archive ends with PK\x05\x06.
    if bytes.len() >= 4 && bytes[..4] == *b"PK\x05\x06" {
        return Kind::Zip;
    }
    // tar: "ustar" at offset 257 (magic compatible with gnu/ustar/posix).
    if bytes.len() >= 262 && (&bytes[257..262] == b"ustar" || &bytes[257..262] == b"ust\x00") {
        return Kind::Tar;
    }
    Kind::Unknown
}

fn sniff(path: &Path) -> io::Result<Kind> {
    let mut f = fs::File::open(path)?;
    let mut buf = [0u8; 512];
    let n = f.read(&mut buf)?;
    Ok(kind_of(&buf[..n]))
}

fn io_err(msg: impl Into<String>) -> io::Error {
    io::Error::other(msg.into())
}

/// Sanitize entry names: reject absolute paths and `..` escapes; return a relative path.
fn safe_join(dest: &Path, name: &str) -> io::Result<PathBuf> {
    let p = Path::new(name);
    if p.is_absolute() {
        return Err(io_err(format!(
            "archive entry escapes dest (absolute): {name}"
        )));
    }
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::ParentDir => {
                return Err(io_err(format!("archive entry escapes dest (..): {name}")));
            }
            _ => {}
        }
    }
    Ok(dest.join(out))
}

/// Extract a tar stream into dest (mirrors Go `Extractor.Extract`: only files are written;
/// directories are created implicitly by the parent dirs of file entries).
fn unpack_tar<R: Read>(reader: R, dest: &Path) -> io::Result<()> {
    let mut ar = tar::Archive::new(reader);
    let entries = ar.entries()?;
    for entry in entries {
        let mut entry = entry?;
        let t = entry.header().entry_type();
        // Only regular files are handled; directory, symlink, and hard-link entries are skipped,
        // the same semantics as Go.
        if !t.is_file() {
            continue;
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        let mode = entry.header().mode()?;
        let target = safe_join(dest, &name)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&target)?;
        io::copy(&mut entry, &mut out)?;
        set_mode(&target, mode);
    }
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o7777));
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}

/// Extract a zip into dest (mirrors the file semantics of Go `Extractor.Extract`).
fn unpack_zip(path: &Path, dest: &Path) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let target = safe_join(dest, &name)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&target)?;
        io::copy(&mut entry, &mut out)?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            set_mode(&target, mode);
        }
    }
    Ok(())
}

/// Open a single-stream decompression reader (the decoder matching the magic number).
fn decompress_reader(kind: Kind, f: fs::File) -> io::Result<Box<dyn io::Read>> {
    match kind {
        Kind::Gzip => Ok(Box::new(flate2::read::MultiGzDecoder::new(f))),
        Kind::Bzip2 => Ok(Box::new(bzip2::read::BzDecoder::new(f))),
        Kind::Xz => Ok(Box::new(xz2::read::XzDecoder::new(f))),
        Kind::Zstd => Ok(Box::new(zstd::stream::read::Decoder::new(f)?)),
        _ => Err(io_err("not a single-stream compressed file")),
    }
}

/// Strip the final extension from a file name (mirrors Go `baseName`).
fn base_name(src: &Path) -> String {
    let name = src
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    match name.rfind('.') {
        Some(i) if i > 0 => name[..i].to_string(),
        _ => name,
    }
}

/// Single-stream compression → decode into dest/temp, then re-sniff: archives are extracted
/// further; otherwise the file is copied/renamed as a single file.
/// Mirrors Go `Extractor.Unarchive` and `vmrUnarchive(src, dst, name, isExe)`.
pub fn unarchive(
    src: &Path,
    dest: &Path,
    single_file_name: Option<&str>,
    is_single_executable: bool,
) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    let kind = sniff(src)?;
    match kind {
        Kind::Zip => unpack_zip(src, dest),
        Kind::Tar => {
            let f = fs::File::open(src)?;
            unpack_tar(f, dest)
        }
        Kind::Gzip | Kind::Bzip2 | Kind::Xz | Kind::Zstd => {
            let tmp_dir = dest.join("temp");
            fs::create_dir_all(&tmp_dir)?;
            let result = (|| -> io::Result<()> {
                let f = fs::File::open(src)?;
                let decoded_name = tmp_dir.join(base_name(src));
                {
                    let mut reader = decompress_reader(kind, f)?;
                    let mut out = fs::File::create(&decoded_name)?;
                    io::copy(&mut reader, &mut out)?;
                }
                let kind2 = sniff(&decoded_name)?;
                match kind2 {
                    Kind::Zip => unpack_zip(&decoded_name, dest),
                    Kind::Tar => {
                        let f = fs::File::open(&decoded_name)?;
                        unpack_tar(f, dest)
                    }
                    _ => {
                        // A compressed single file (the Go "no formats" branch). Target name:
                        // an explicit rename wins; otherwise the source file name with its last
                        // extension removed (mirrors Go baseName).
                        let name = single_file_name
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| base_name(src));
                        let dst = dest.join(name);
                        fs::copy(&decoded_name, &dst)?;
                        if is_single_executable {
                            add_exec_bit(&dst);
                        }
                        Ok(())
                    }
                }
            })();
            let _ = fs::remove_dir_all(&tmp_dir);
            result
        }
        Kind::Unknown => Err(io_err(format!(
            "unsupported or unrecognized archive format: {}",
            src.display()
        ))),
    }
}

/// Add executable bits (mirrors Go: `mode | 0111`; existing bits are not filtered).
#[cfg(unix)]
fn add_exec_bit(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mode = meta.permissions().mode();
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o111));
    }
}

#[cfg(not(unix))]
fn add_exec_bit(_path: &Path) {}

fn is_tar_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".tar")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tar.xz")
        || lower.ends_with(".tar.zst")
        || lower.ends_with(".tgz")
}

/// Expand nested `.zip` files found at the top level of `destDir` into destDir (mirrors Go
/// `handleMultiCompress`: only non-directory entries at destDir's top level are scanned).
fn expand_nested_zips(dest_dir: &Path) {
    let Ok(entries) = fs::read_dir(dest_dir) else {
        return;
    };
    for e in entries.flatten() {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            continue;
        }
        let name = e.file_name();
        if name.to_string_lossy().ends_with(".zip") {
            let _ = unpack_zip(&e.path(), dest_dir);
        }
    }
}

/// Extraction entry point (mirrors Go `utils.Extract`): system commands take priority; on failure
/// it falls back to the library implementation.
///
/// System commands: `.zip` → `unzip <src> -d <dest>`; `.tar*` → `tar -xf <src> -C <dest>`
/// (on Windows the zip case uses the powershell `expand -r` branch, mirroring Go's `Unzip`).
pub fn extract(src: &Path, dest_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dest_dir)?;
    let name = src
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lower = name.to_lowercase();

    let sys_ok = if lower.ends_with(".zip") {
        #[cfg(windows)]
        {
            // exec wraps cmd /c internally (mirrors Go ExecuteSysCommand); System32 is prepended
            // to PATH so that powershell can be resolved.
            let path = std::env::var("PATH").unwrap_or_default();
            let prefixed = format!(r"C:\Windows\System32;{path}");
            std::env::set_var("PATH", prefixed);
            exec(
                true,
                "",
                &[
                    "powershell".to_string(),
                    "expand".to_string(),
                    "-r".to_string(),
                    src.display().to_string(),
                    dest_dir.display().to_string(),
                ],
            )
            .is_ok()
        }
        #[cfg(not(windows))]
        {
            exec(
                true,
                "",
                &[
                    "unzip".to_string(),
                    src.display().to_string(),
                    "-d".to_string(),
                    dest_dir.display().to_string(),
                ],
            )
            .is_ok()
        }
    } else if is_tar_name(&name) {
        exec(
            true,
            "",
            &[
                "tar".to_string(),
                "-xf".to_string(),
                src.display().to_string(),
                "-C".to_string(),
                dest_dir.display().to_string(),
            ],
        )
        .is_ok()
    } else {
        false
    };

    if sys_ok {
        expand_nested_zips(dest_dir);
        return Ok(());
    }
    // Library-implementation fallback: a plain extraction via unarchive with no rename or
    // executable-bit semantics.
    unarchive(src, dest_dir, None, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let p = std::env::temp_dir()
                .join(format!("vmr-utils-extract-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn make_zip(path: &Path, entries: &[(&str, &str)]) {
        let f = fs::File::create(path).unwrap();
        let mut w = zip::ZipWriter::new(f);
        for (name, content) in entries {
            w.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            w.write_all(content.as_bytes()).unwrap();
        }
        w.finish().unwrap();
    }

    fn make_tar_gz(path: &Path, entries: &[(&str, &str)]) {
        let f = fs::File::create(path).unwrap();
        let enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
        let mut w = tar::Builder::new(enc);
        for (name, content) in entries {
            let mut hdr = tar::Header::new_gnu();
            hdr.set_size(content.len() as u64);
            hdr.set_mode(0o755);
            hdr.set_cksum();
            w.append_data(&mut hdr, *name, content.as_bytes()).unwrap();
        }
        w.into_inner().unwrap().finish().unwrap();
    }

    #[test]
    fn unzips_with_nested_dirs() {
        let t = TempDir::new("zip");
        let src = t.path().join("a.zip");
        make_zip(
            &src,
            &[("pkg/bin/tool", "tool-content"), ("pkg/readme", "hi")],
        );
        let dest = t.path().join("out");
        unarchive(&src, &dest, None, false).unwrap();
        assert_eq!(
            fs::read_to_string(dest.join("pkg/bin/tool")).unwrap(),
            "tool-content"
        );
        assert_eq!(fs::read_to_string(dest.join("pkg/readme")).unwrap(), "hi");
    }

    #[test]
    fn unpacks_tar_gz() {
        let t = TempDir::new("targz");
        let src = t.path().join("b.tar.gz");
        make_tar_gz(&src, &[("lib/x.so", "elf"), ("bin/go", "go")]);
        let dest = t.path().join("out");
        unarchive(&src, &dest, None, false).unwrap();
        assert_eq!(fs::read_to_string(dest.join("bin/go")).unwrap(), "go");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dest.join("bin/go"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755, "tar 文件模式生效");
        }
    }

    #[test]
    fn compressed_single_file_rename_and_exec() {
        let t = TempDir::new("single-gz");
        let src = t.path().join("tool.gz");
        let f = fs::File::create(&src).unwrap();
        let mut enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
        enc.write_all(b"#!/bin/sh\necho hi\n").unwrap();
        enc.finish().unwrap();

        let dest = t.path().join("out");
        unarchive(&src, &dest, Some("renamed-tool"), true).unwrap();
        let out = dest.join("renamed-tool");
        assert_eq!(fs::read_to_string(&out).unwrap(), "#!/bin/sh\necho hi\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&out).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "可执行位已补");
        }
        // The temp staging directory has been cleaned up.
        assert!(!dest.join("temp").exists());
    }

    #[test]
    fn rejects_path_traversal_entries() {
        // The tar builder itself rejects `..` names, so the guard is exercised with zip,
        // which allows arbitrary names.
        let t = TempDir::new("traversal");
        let src = t.path().join("evil.zip");
        make_zip(&src, &[("../evil", "pwned"), ("ok.txt", "fine")]);
        let dest = t.path().join("out");
        assert!(unarchive(&src, &dest, None, false).is_err());
        assert!(!t.path().join("evil").exists());
        // The guard fires at the first escaping entry, so no legitimate content is written down
        // (an extra safety guarantee on top of Go's short-circuit semantics).
        assert!(!dest.exists() || !dest.join("ok.txt").exists());
    }

    #[test]
    #[cfg(not(windows))] // requires the unzip system command (common on unix; windows uses powershell)
    fn extract_via_system_command_expands_nested_zip() {
        // The system-unzip preferred path: the outer zip embeds a zip file at its top level.
        let t = TempDir::new("syszip");
        // Build the inner zip bytes.
        let inner_path = t.path().join("inner.zip");
        make_zip(&inner_path, &[("inner.txt", "nested-content")]);
        let inner_bytes = fs::read(&inner_path).unwrap();

        let outer = t.path().join("outer.zip");
        {
            let f = fs::File::create(&outer).unwrap();
            let mut w = zip::ZipWriter::new(f);
            w.start_file("inner.zip", zip::write::SimpleFileOptions::default())
                .unwrap();
            w.write_all(&inner_bytes).unwrap();
            w.finish().unwrap();
        }
        let dest = t.path().join("out");
        extract(&outer, &dest).unwrap();
        // handleMultiCompress: the top-level nested zip is expanded into the dest root.
        assert_eq!(
            fs::read_to_string(dest.join("inner.txt")).unwrap_or_default(),
            "nested-content"
        );
    }

    #[test]
    fn library_fallback_on_gz_single_file() {
        let t = TempDir::new("fallback-gz");
        let src = t.path().join("data.raw.gz");
        let f = fs::File::create(&src).unwrap();
        let mut enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
        enc.write_all(b"raw-bytes").unwrap();
        enc.finish().unwrap();
        let dest = t.path().join("out");
        fs::create_dir_all(&dest).unwrap();
        // System commands do not support a bare .gz (not .tar.gz) → the library path extracts it
        // as a single file.
        extract(&src, &dest).unwrap();
        assert_eq!(
            fs::read_to_string(dest.join("data.raw")).unwrap(),
            "raw-bytes"
        );
    }
}
