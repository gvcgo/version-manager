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
