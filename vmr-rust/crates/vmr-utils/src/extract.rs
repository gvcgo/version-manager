//! 压缩/归档解压（对应 plan.md §3.2 `extract.rs`）。
//!
//! 对齐 Go 两个入口：
//! - `unarchive`：对应 `internal/utils/extract/extractor.go` 的 `Extractor.Unarchive`，
//!   是 Lua `vmrUnarchive` 的落地——按**魔数**识别格式；单流压缩（gz/bz2/xz/zst）
//!   先解到 `<dest>/temp/`，再识别：归档则解入 dest；否则视为"压缩单文件"，
//!   复制/改名到 dest，`single_exe` 时补 `0111` 执行位。
//! - `extract`：对应 `internal/utils/extractor.go` 的 `Extract`——**优先系统命令**
//!   （`.zip`→`unzip -d`，`.tar*`→`tar -xf -C`，对齐 Go），成功后展开顶层
//!   嵌套 `.zip`（`handleMultiCompress`）；系统命令不可用/失败回退库实现。
//!
//! 行为对齐 Go 的 quirk：归档内**目录项不单独创建**（由文件项的父目录 MkdirAll
//! 隐式创建 → 空目录丢弃）。与 Go 差异（安全加固，文档化）：拒绝 `..`/绝对路径
//! 条目（zip-slip 防护），Go 直接 Join 存在越界风险；归档符号链接条目跳过。

use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::exec::exec;

/// 归档/压缩格式（魔数识别，等价 mholt/archives Identify）。
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
    // zip 空归档以 PK\x05\x06 结尾。
    if bytes.len() >= 4 && bytes[..4] == *b"PK\x05\x06" {
        return Kind::Zip;
    }
    // tar：偏移 257 处 "ustar"（magic 兼容 gnu/ustar/posix）。
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

/// 条目名安全化：拒绝绝对路径与 `..` 逃逸；返回相对路径。
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

/// 解 tar 流到 dest（对齐 Go `Extractor.Extract`：只写文件，目录由父级隐式创建）。
fn unpack_tar<R: Read>(reader: R, dest: &Path) -> io::Result<()> {
    let mut ar = tar::Archive::new(reader);
    let entries = ar.entries()?;
    for entry in entries {
        let mut entry = entry?;
        let t = entry.header().entry_type();
        // 只处理普通文件（目录、符号链接、硬链接等条目跳过，Go 同款语义）。
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

/// 解 zip 到 dest（对齐 Go `Extractor.Extract` 的文件语义）。
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

/// 打开单流解压 reader（魔数对应的解码器）。
fn decompress_reader(kind: Kind, f: fs::File) -> io::Result<Box<dyn io::Read>> {
    match kind {
        Kind::Gzip => Ok(Box::new(flate2::read::MultiGzDecoder::new(f))),
        Kind::Bzip2 => Ok(Box::new(bzip2::read::BzDecoder::new(f))),
        Kind::Xz => Ok(Box::new(xz2::read::XzDecoder::new(f))),
        Kind::Zstd => Ok(Box::new(zstd::stream::read::Decoder::new(f)?)),
        _ => Err(io_err("not a single-stream compressed file")),
    }
}

/// 文件名去最后一个扩展名（对齐 Go `baseName`）。
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

/// 单流压缩 → 解到 dest/temp；再识别：归档继续解，否则按单文件复制/改名。
/// 对齐 Go `Extractor.Unarchive` 与 `vmrUnarchive(src, dst, name, isExe)`。
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
                        // 压缩单文件（Go："no formats" 分支）。目标名：
                        // 显式改名优先，否则取源文件名去最后一个扩展名（对齐 Go baseName）。
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

/// 追加可执行位（对齐 Go：`mode | 0111`，不过滤已有位）。
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

/// 顶层 `destDir` 内的嵌套 `.zip` 展开到 destDir（对齐 Go `handleMultiCompress`：
/// 只扫描 destDir 顶层非目录条目）。
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

/// 解压入口（对齐 Go `utils.Extract`）：系统命令优先，失败回退库实现。
///
/// 系统命令：`.zip` → `unzip <src> -d <dest>`；`.tar*` → `tar -xf <src> -C <dest>`
/// （Windows 下 zip 用 powershell `expand -r`，对齐 Go `Unzip` 分支）。
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
            // exec 内部会包 cmd /c（对齐 Go ExecuteSysCommand）；PATH 前置
            // System32 保证 powershell 可解析。
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
    // 库实现回退（unarchive 无改名/无执行位语义的纯解压）。
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
        // temp 中转目录已清理。
        assert!(!dest.join("temp").exists());
    }

    #[test]
    fn rejects_path_traversal_entries() {
        // tar builder 本身拒绝 .. 名，用 zip（允许任意名）验证防护。
        let t = TempDir::new("traversal");
        let src = t.path().join("evil.zip");
        make_zip(&src, &[("../evil", "pwned"), ("ok.txt", "fine")]);
        let dest = t.path().join("out");
        assert!(unarchive(&src, &dest, None, false).is_err());
        assert!(!t.path().join("evil").exists());
        // 防护发生在首个越界条目，合法内容不落盘（保持 Go 短路语义之外的安全保证）。
        assert!(!dest.exists() || !dest.join("ok.txt").exists());
    }

    #[test]
    #[cfg(not(windows))] // 依赖 unzip 系统命令（unix 常见；windows 分支走 powershell）
    fn extract_via_system_command_expands_nested_zip() {
        // 系统 unzip 优先路径：外层 zip 顶层内嵌一个 zip 文件。
        let t = TempDir::new("syszip");
        // 构造内层 zip 字节。
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
        // handleMultiCompress：顶层嵌套 zip 被展开到 dest 根。
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
        // 系统命令不支持裸 .gz（非 .tar.gz）→ 库路径解压为单文件。
        extract(&src, &dest).unwrap();
        assert_eq!(
            fs::read_to_string(dest.join("data.raw")).unwrap(),
            "raw-bytes"
        );
    }
}
