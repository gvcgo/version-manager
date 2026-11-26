//! utils bindings (24 vmr* general-purpose functions, mirroring Go `lua_global/utils.go`).

use std::path::Path;

use mlua::{Lua, Table, Value};

use crate::bindings::register_fn;

/// Process os/arch (Go naming: darwin/windows/linux + amd64/arm64/386).
pub fn os_arch() -> (String, String) {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        other => other,
    };
    (os.to_string(), arch.to_string())
}

fn argv(table: Option<Table>) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(t) = table {
        for v in t.sequence_values::<Value>().flatten() {
            out.push(crate::req::str_of(&v));
        }
    }
    out
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(lua, "vmrGetOsArch", |_, ()| {
        let (os, arch) = os_arch();
        Ok((os, arch))
    })?;

    register_fn(lua, "vmrHasAvx2", |_, ()| {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let supported = std::is_x86_feature_detected!("avx2");
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let supported = false;
        Ok(supported)
    })?;

    register_fn(
        lua,
        "vmrRegexpFindString",
        |_, (pat, content): (String, String)| {
            let out = regex::Regex::new(&pat)
                .ok()
                .and_then(|re| re.find(&content))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            Ok(out)
        },
    )?;

    register_fn(lua, "vmrHasPrefix", |_, (s, p): (String, String)| {
        Ok(s.starts_with(&p))
    })?;
    register_fn(lua, "vmrHasSuffix", |_, (s, p): (String, String)| {
        Ok(s.ends_with(&p))
    })?;
    register_fn(lua, "vmrContains", |_, (s, p): (String, String)| {
        Ok(s.contains(&p))
    })?;
    register_fn(lua, "vmrTrimPrefix", |_, (s, p): (String, String)| {
        Ok(s.trim_start_matches(&p).to_string())
    })?;
    register_fn(lua, "vmrTrimSuffix", |_, (s, p): (String, String)| {
        Ok(s.trim_end_matches(&p).to_string())
    })?;
    register_fn(lua, "vmrTrim", |_, (s, cut): (String, String)| {
        Ok(s.trim_matches(|c| cut.contains(c)).to_string())
    })?;
    register_fn(lua, "vmrTrimSpace", |_, (s,): (String,)| {
        Ok(s.trim().to_string())
    })?;
    register_fn(lua, "vmrToLower", |_, (s,): (String,)| Ok(s.to_lowercase()))?;

    register_fn(lua, "vmrSplit", |lua, (content, sep): (String, String)| {
        let items: Vec<String> = if content.is_empty() || sep.is_empty() {
            Vec::new()
        } else {
            content.split(&sep).map(|s| s.to_string()).collect()
        };
        let t = lua.create_table()?;
        for it in &items {
            t.push(it.clone())?;
        }
        let n = items.len();
        Ok((Value::Table(t), n as i64))
    })?;

    register_fn(
        lua,
        "vmrSprintf",
        |_, (pattern, array): (String, Option<Table>)| {
            let args = argv(array);
            Ok(sprintf_s(&pattern, &args))
        },
    )?;

    register_fn(lua, "vmrUrlJoin", |_, (base, path): (String, String)| {
        Ok(url_join(&base, &path))
    })?;
    register_fn(lua, "vmrPathJoin", |_, (base, path): (String, String)| {
        if base.is_empty() || path.is_empty() {
            return Ok(String::new());
        }
        let p = Path::new(&base).join(&path);
        Ok(p.to_string_lossy().into_owned())
    })?;
    register_fn(lua, "vmrLenString", |_, (s,): (String,)| Ok(s.len() as i64))?;
    register_fn(lua, "vmrGetOsEnv", |_, (key,): (String,)| {
        let v = if key.is_empty() {
            String::new()
        } else {
            std::env::var(&key).unwrap_or_default()
        };
        Ok(v)
    })?;
    register_fn(lua, "vmrSetOsEnv", |_, (key, value): (String, String)| {
        // Mirror Go: Setenv returns false on error (the Rust env-immutable case is extremely rare).
        let ok = std::panic::catch_unwind(|| {
            unsafe { std::env::set_var(&key, &value) };
        })
        .is_ok();
        Ok(ok)
    })?;

    register_fn(
        lua,
        "vmrExecSystemCmd",
        |_, (collect, workdir, args): (bool, String, Option<Table>)| {
            let args = argv(args);
            let out = vmr_utils::exec::exec(collect, &workdir, &args).unwrap_or_default();
            // Go semantics: (stdout, ok). stdout is an empty string when not collecting.
            Ok((out, true))
        },
    )?;
    // The above ok-always-true diverges from Go → switch to the error-propagating version below.
    // vmrExecSystemCmd needs the error bit, so this overrides it with the real implementation:
    lua.globals().set(
        "vmrExecSystemCmd",
        lua.create_function(
            |_, (collect, workdir, args): (bool, String, Option<Table>)| {
                let args = argv(args);
                let out = vmr_utils::exec::exec(collect, &workdir, &args);
                match out {
                    Ok(s) => Ok((s, true)),
                    Err(_) => Ok((String::new(), false)),
                }
            },
        )?,
    )?;

    register_fn(lua, "vmrReadFile", |_, (path,): (String,)| {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        Ok(content)
    })?;
    register_fn(
        lua,
        "vmrWriteFile",
        |_, (path, content): (String, String)| {
            if path.is_empty() {
                return Ok(false);
            }
            Ok(std::fs::write(&path, content).is_ok())
        },
    )?;
    register_fn(lua, "vmrCopyFile", |_, (src, dst): (String, String)| {
        Ok(vmr_utils::copy::copy_file(Path::new(&src), Path::new(&dst)).is_ok())
    })?;
    register_fn(lua, "vmrCopyDir", |_, (src, dst): (String, String)| {
        Ok(vmr_utils::copy::copy_directory(Path::new(&src), Path::new(&dst)).is_ok())
    })?;
    register_fn(lua, "vmrDownloadFile", |_, (url, destination): (String, String)| {
        Ok(download_file(&url, Path::new(&destination)))
    })?;
    register_fn(lua, "vmrChmodExecutable", |_, (path,): (String,)| {
        Ok(chmod_executable(Path::new(&path)))
    })?;
    register_fn(lua, "vmrSymlink", |_, (target, link): (String, String)| {
        Ok(symlink(Path::new(&target), Path::new(&link)))
    })?;
    register_fn(lua, "vmrFileExists", |_, (path,): (String,)| {
        Ok(!path.is_empty() && Path::new(&path).exists())
    })?;
    register_fn(lua, "vmrRemoveFile", |_, (path,): (String,)| {
        Ok(!path.is_empty() && std::fs::remove_file(path).is_ok())
    })?;
    register_fn(lua, "vmrExtract", |_, (src, dst): (String, String)| {
        Ok(!src.is_empty() && !dst.is_empty()
            && vmr_utils::extract::extract(Path::new(&src), Path::new(&dst)).is_ok())
    })?;
    register_fn(lua, "vmrFindHome", |_, (dir, flags, except_dir): (String, Table, bool)| {
        let mut finder = vmr_utils::find_dir::HomeDirFinder::new(argv(Some(flags)));
        finder.set_flag_dir_excepted(except_dir);
        finder.find(Path::new(&dir));
        Ok(finder.get_dir_name().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default())
    })?;
    register_fn(lua, "vmrHardLink", |_, (target, link): (String, String)| {
        Ok(std::fs::hard_link(target, link).is_ok())
    })?;
    register_fn(lua, "vmrFindFile", |_, (root, name, depth): (String, String, usize)| {
        Ok(find_file(Path::new(&root), &name, depth)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default())
    })?;
    register_fn(lua, "vmrChmodDirFiles", |_, (dir,): (String,)| {
        Ok(chmod_dir_files(Path::new(&dir)))
    })?;
    register_fn(lua, "vmrReadDir", |lua, (path,): (String,)| {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };
        let table = lua.create_table()?;
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return Ok(None),
            };
            table.push(entry.file_name().to_string_lossy().as_ref())?;
        }
        Ok(Some(table))
    })?;
    register_fn(lua, "vmrTempDir", |_, ()| {
        Ok(temp_dir().unwrap_or_default())
    })?;
    register_fn(lua, "vmrExecSystemCmdEnv", |_, (workdir, args, env): (String, Table, Table)| {
        let args = argv(Some(args));
        let mut command = match args.split_first() {
            Some((program, args)) => {
                let mut command = std::process::Command::new(program);
                command.args(args);
                command
            }
            None => return Ok(false),
        };
        if !workdir.is_empty() {
            command.current_dir(workdir);
        }
        for pair in env.pairs::<String, String>() {
            let (key, value) = pair?;
            command.env(key, value);
        }
        Ok(command.status().map(|s| s.success()).unwrap_or(false))
    })?;
    register_fn(lua, "vmrExecDirect", |_, (collect, workdir, args, raw): (bool, String, Table, Option<String>)| {
        Ok(exec_direct(collect, &workdir, argv(Some(args)), raw.as_deref()))
    })?;
    register_fn(lua, "vmrCanonicalPath", |_, (path,): (String,)| {
        Ok(std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default())
    })?;
    register_fn(lua, "vmrCreateDir", |_, (dir,): (String,)| {
        if dir.is_empty() {
            return Ok(false);
        }
        Ok(std::fs::create_dir_all(&dir).is_ok())
    })?;
    register_fn(lua, "vmrRemoveAll", |_, (dir,): (String,)| {
        if dir.is_empty() {
            return Ok(false);
        }
        Ok(std::fs::remove_dir_all(&dir).is_ok() || !std::path::Path::new(&dir).exists())
    })?;

    // extractor: vmrUnarchive (corresponds to vmr_utils extract::unarchive).
    register_fn(
        lua,
        "vmrUnarchive",
        |_, (src, dst, single_name, single_exe): (String, String, String, bool)| {
            if src.is_empty() || dst.is_empty() {
                return Ok(false);
            }
            let name = if single_name.is_empty() {
                None
            } else {
                Some(single_name.as_str())
            };
            Ok(
                vmr_utils::extract::unarchive(Path::new(&src), Path::new(&dst), name, single_exe)
                    .is_ok(),
            )
        },
    )?;
    Ok(())
}

fn download_file(url: &str, destination: &Path) -> bool {
    if url.is_empty() || destination.as_os_str().is_empty() {
        return false;
    }
    let (mapped, proxy) = vmr_net::fetcher::resolve_chain(url);
    let response = vmr_net::fetcher::Fetcher::new(proxy)
        .ok()
        .and_then(|client| client.client().get(&mapped).send().ok())
        .and_then(|response| response.error_for_status().ok())
        .or_else(|| {
            if mapped == url {
                return None;
            }
            vmr_net::fetcher::Fetcher::new(None)
                .ok()
                .and_then(|client| client.client().get(url).send().ok())
                .and_then(|response| response.error_for_status().ok())
        });
    let Some(mut response) = response else {
        return false;
    };
    if let Some(parent) = destination.parent().filter(|p| !p.as_os_str().is_empty()) {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    let Ok(mut file) = std::fs::OpenOptions::new().write(true).create_new(true).open(destination) else {
        return false;
    };
    if std::io::copy(&mut response, &mut file).is_err() {
        drop(file);
        let _ = std::fs::remove_file(destination);
        return false;
    }
    true
}

fn chmod_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let Ok(metadata) = std::fs::metadata(path) else {
            return false;
        };
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        std::fs::set_permissions(path, permissions).is_ok()
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn symlink(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        let resolved = if target.is_absolute() {
            target.to_path_buf()
        } else {
            link.parent().unwrap_or(Path::new("")).join(target)
        };
        if resolved.is_dir() {
            std::os::windows::fs::symlink_dir(target, link).is_ok()
        } else {
            std::os::windows::fs::symlink_file(target, link).is_ok()
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, link);
        false
    }
}

fn find_file(root: &Path, name: &str, depth: usize) -> Option<std::path::PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let ty = entry.file_type().ok()?;
        if ty.is_file() && entry.file_name() == name {
            return Some(entry.path());
        }
        if depth > 0 && ty.is_dir() {
            if let Some(path) = find_file(&entry.path(), name, depth - 1) {
                return Some(path);
            }
        }
    }
    None
}

fn chmod_dir_files(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries {
        let Ok(entry) = entry else { return false };
        let Ok(ty) = entry.file_type() else { return false };
        if ty.is_file() && !chmod_executable(&entry.path()) {
            return false;
        }
    }
    true
}

fn temp_dir() -> Option<String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    for _ in 0..100 {
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("vmr-{}-{id}", std::process::id()));
        match std::fs::create_dir(&path) {
            Ok(()) => return Some(path.to_string_lossy().into_owned()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
    None
}

fn exec_direct(collect: bool, workdir: &str, args: Vec<String>, raw: Option<&str>) -> (String, bool) {
    let Some((program, args)) = args.split_first() else {
        return (String::new(), false);
    };
    let mut command = std::process::Command::new(program);
    command.args(args);
    if let Some(raw) = raw {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.raw_arg(raw);
        }
        #[cfg(not(windows))]
        command.arg(raw);
    }
    if !workdir.is_empty() {
        command.current_dir(workdir);
    }
    if collect {
        command.stderr(std::process::Stdio::inherit());
        match command.output() {
            Ok(output) => (String::from_utf8_lossy(&output.stdout).into_owned(), output.status.success()),
            Err(_) => (String::new(), false),
        }
    } else {
        (String::new(), command.status().map(|s| s.success()).unwrap_or(false))
    }
}

/// Sequential %s substitution (the common fmt.Sprintf("%s") pattern plugins use).
fn sprintf_s(pattern: &str, args: &[String]) -> String {
    if args.is_empty() {
        return pattern.to_string();
    }
    let mut out = String::new();
    let mut rest = pattern;
    let mut i = 0;
    loop {
        match rest.find("%s") {
            Some(pos) => {
                out.push_str(&rest[..pos]);
                if i < args.len() {
                    out.push_str(&args[i]);
                    i += 1;
                } else {
                    out.push_str("%s");
                }
                rest = &rest[pos + 2..];
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

/// URL concatenation (the common shape of Go url.JoinPath).
fn url_join(base: &str, path: &str) -> String {
    if base.is_empty() {
        return String::new();
    }
    let b = base.trim_end_matches('/');
    let p = path.trim_start_matches('/');
    if p.is_empty() {
        return base.to_string();
    }
    format!("{b}/{p}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_helpers() {
        let dir = temp_dir().unwrap();
        let root = Path::new(&dir);
        let file = root.join("file");
        std::fs::write(&file, "data").unwrap();
        assert!(chmod_executable(&file));
        assert_eq!(find_file(root, "file", 0), Some(file.clone()));
        assert!(symlink(Path::new("file"), &root.join("link")) || cfg!(windows));
        let lua = Lua::new();
        register(&lua).unwrap();
        let exists: mlua::Function = lua.globals().get("vmrFileExists").unwrap();
        assert!(exists.call::<bool>(file.to_string_lossy().as_ref()).unwrap());
        assert!(!exists.call::<bool>("").unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn download_http_status_and_partial_cleanup() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let dir = temp_dir().unwrap();
        let destination = Path::new(&dir).join("download");
        for response in [
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nshort",
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            });
            assert!(!download_file(&format!("http://{addr}/file"), &destination));
            server.join().unwrap();
            assert!(!destination.exists());
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn url_join_shapes() {
        assert_eq!(
            url_join("https://go.dev", "/dl/x.tar.gz"),
            "https://go.dev/dl/x.tar.gz"
        );
        assert_eq!(url_join("https://a.com/b/", "c"), "https://a.com/b/c");
        assert_eq!(url_join("https://a.com/b", ""), "https://a.com/b");
        assert_eq!(url_join("", "x"), "");
    }

    #[test]
    fn sprintf_basic() {
        assert_eq!(sprintf_s("v%s.%s", &["1".into(), "2".into()]), "v1.2");
        assert_eq!(sprintf_s("no args", &[]), "no args");
    }

    #[test]
    fn os_arch_names() {
        let (os, arch) = os_arch();
        assert!(matches!(os.as_str(), "linux" | "darwin" | "windows"));
        assert!(matches!(arch.as_str(), "amd64" | "arm64" | "386"));
    }
}
