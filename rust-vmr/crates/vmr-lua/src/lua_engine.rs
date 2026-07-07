use mlua::{Lua, Table, Value};
use crate::types::*;

pub struct LuaEngine {
    pub lua: Lua,
}

impl LuaEngine {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let engine = LuaEngine { lua };
        engine.register_globals()?;
        Ok(engine)
    }

    fn register_globals(&self) -> mlua::Result<()> {
        let globals = self.lua.globals();

        // === HTTP Request ===
        // vmrGetResponse(url, timeout, headers_table) -> string
        let get_response = self.lua.create_function(
            |_lua, (url, timeout, headers): (String, Option<i64>, Option<Table>)| {
                let timeout_secs = timeout
                    .filter(|&t| t > 0)
                    .unwrap_or(180)
                    .max(1) as u64;

                let mut req_headers = reqwest::header::HeaderMap::new();
                if let Some(ht) = headers {
                    for pair in ht.pairs::<String, String>() {
                        if let Ok((k, v)) = pair {
                            if let Ok(name) =
                                reqwest::header::HeaderName::from_bytes(k.as_bytes())
                            {
                                if let Ok(val) = reqwest::header::HeaderValue::from_str(&v) {
                                    req_headers.insert(name, val);
                                }
                            }
                        }
                    }
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(timeout_secs))
                    .build()
                    .unwrap_or_default();

                match client.get(&url).headers(req_headers).send() {
                    Ok(resp) => {
                        if resp.status().as_u16() == 200 {
                            let body = resp.text().unwrap_or_default();
                            if !body.is_empty() {
                                return Ok(body);
                            }
                        }
                        Ok(String::new())
                    }
                    Err(_) => Ok(String::new()),
                }
            },
        )?;
        globals.set("vmrGetResponse", get_response)?;

        // === OS/Arch ===
        let get_os_arch = self.lua.create_function(|_, ()| {
            Ok((std::env::consts::OS, std::env::consts::ARCH))
        })?;
        globals.set("vmrGetOsArch", get_os_arch)?;

        // === String utilities ===
        globals.set(
            "vmrHasPrefix",
            self.lua
                .create_function(|_, (s, prefix): (String, String)| Ok(s.starts_with(&prefix)))?,
        )?;
        globals.set(
            "vmrHasSuffix",
            self.lua
                .create_function(|_, (s, suffix): (String, String)| Ok(s.ends_with(&suffix)))?,
        )?;
        globals.set(
            "vmrContains",
            self.lua
                .create_function(|_, (s, sub): (String, String)| Ok(s.contains(&sub)))?,
        )?;
        globals.set(
            "vmrTrimPrefix",
            self.lua.create_function(|_, (s, prefix): (String, String)| {
                Ok(s.strip_prefix(&prefix).unwrap_or(&s).to_string())
            })?,
        )?;
        globals.set(
            "vmrTrimSuffix",
            self.lua.create_function(|_, (s, suffix): (String, String)| {
                Ok(s.strip_suffix(&suffix).unwrap_or(&s).to_string())
            })?,
        )?;
        globals.set(
            "vmrTrim",
            self.lua.create_function(|_, (s, cut): (String, String)| {
                Ok(s.trim_matches(|c: char| cut.contains(c)).to_string())
            })?,
        )?;
        globals.set(
            "vmrTrimSpace",
            self.lua
                .create_function(|_, s: String| Ok(s.trim().to_string()))?,
        )?;
        globals.set(
            "vmrToLower",
            self.lua
                .create_function(|_, s: String| Ok(s.to_lowercase()))?,
        )?;
        globals.set(
            "vmrLenString",
            self.lua
                .create_function(|_, s: String| Ok(s.len() as i64))?,
        )?;

        // vmrSplit(content, sep) -> (table, len)
        globals.set(
            "vmrSplit",
            self.lua.create_function(
                |lua, (content, sep): (String, String)| {
                    let parts: Vec<String> = content.split(&sep).map(|s| s.to_string()).collect();
                    let len = parts.len();
                    let table = lua.create_table_from(
                        parts.into_iter().enumerate().map(|(i, v)| (i + 1, v)),
                    )?;
                    Ok((table, len as i64))
                },
            )?,
        )?;

        // vmrSprintf(pattern, args_table) -> string
        globals.set(
            "vmrSprintf",
            self.lua
                .create_function(|_, (pattern, args): (String, mlua::Table)| {
                    let mut arg_strings: Vec<String> = Vec::new();
                    let args_len = args.raw_len();
                    for i in 1..=args_len {
                        if let Ok(v) = args.get::<String>(i) {
                            arg_strings.push(v);
                        }
                    }
                    Ok(format_sprintf(&pattern, &arg_strings))
                })?,
        )?;

        // URL/Path join
        globals.set(
            "vmrUrlJoin",
            self.lua
                .create_function(|_, (base, path): (String, String)| {
                    if base.is_empty() || path.is_empty() {
                        return Ok(String::new());
                    }
                    let base = base.trim_end_matches('/');
                    let path = path.trim_start_matches('/');
                    Ok(format!("{}/{}", base, path))
                })?,
        )?;

        globals.set(
            "vmrPathJoin",
            self.lua
                .create_function(|_, (base, path): (String, String)| {
                    if base.is_empty() || path.is_empty() {
                        return Ok(String::new());
                    }
                    Ok(std::path::Path::new(&base)
                        .join(&path)
                        .to_string_lossy()
                        .to_string())
                })?,
        )?;

        // Environment variables
        globals.set(
            "vmrGetOsEnv",
            self.lua.create_function(|_, key: String| {
                Ok(std::env::var(&key).unwrap_or_default())
            })?,
        )?;
        globals.set(
            "vmrSetOsEnv",
            self.lua.create_function(
                |_, (key, val): (String, String)| {
                    std::env::set_var(&key, &val);
                    Ok(true)
                },
            )?,
        )?;

        // File I/O
        globals.set(
            "vmrReadFile",
            self.lua.create_function(|_, path: String| {
                Ok(std::fs::read_to_string(&path).unwrap_or_default())
            })?,
        )?;
        globals.set(
            "vmrWriteFile",
            self.lua.create_function(
                |_, (path, content): (String, String)| {
                    Ok(std::fs::write(&path, &content).is_ok())
                },
            )?,
        )?;
        globals.set(
            "vmrCreateDir",
            self.lua.create_function(|_, path: String| {
                Ok(std::fs::create_dir_all(&path).is_ok())
            })?,
        )?;
        globals.set(
            "vmrRemoveAll",
            self.lua.create_function(|_, path: String| {
                Ok(std::fs::remove_dir_all(&path).is_ok())
            })?,
        )?;
        globals.set(
            "vmrCopyFile",
            self.lua.create_function(
                |_, (src, dst): (String, String)| {
                    Ok(vmr_utils::fs::copy_file(
                        std::path::Path::new(&src),
                        std::path::Path::new(&dst),
                    ).is_ok())
                },
            )?,
        )?;
        globals.set(
            "vmrCopyDir",
            self.lua.create_function(
                |_, (src, dst): (String, String)| {
                    Ok(vmr_utils::fs::copy_directory(
                        std::path::Path::new(&src),
                        std::path::Path::new(&dst),
                    ).is_ok())
                },
            )?,
        )?;

        // System command execution
        // vmrExecSystemCmd(collect_bool, workdir_str, program_str, arg1, arg2, ...) -> (output, success)
        globals.set(
            "vmrExecSystemCmd",
            self.lua.create_function(
                |lua, params: mlua::MultiValue| {
                    let mut args: Vec<mlua::Value> = params.into_iter().collect();
                    if args.len() < 3 {
                        return Ok((String::new(), false));
                    }
                    // args[0] = collect(bool), args[1] = workdir(str), args[2..] = cmd args
                    let collect: bool = mlua::FromLua::from_lua(args.remove(0), &lua).unwrap_or(true);
                    let workdir: String = mlua::FromLua::from_lua(args.remove(0), &lua).unwrap_or_default();
                    let mut cmd_args: Vec<String> = Vec::new();
                    for a in args {
                        let s: String = mlua::FromLua::from_lua(a, &lua).unwrap_or_default();
                        cmd_args.push(s);
                    }
                    if cmd_args.is_empty() {
                        return Ok((String::new(), false));
                    }
                    let cwd = if workdir.is_empty() {
                        std::env::current_dir().unwrap_or_default()
                    } else {
                        std::path::PathBuf::from(&workdir)
                    };
                    let output = std::process::Command::new(&cmd_args[0])
                        .args(&cmd_args[1..])
                        .current_dir(&cwd)
                        .output();
                    match output {
                        Ok(o) => {
                            let stdout =
                                String::from_utf8_lossy(&o.stdout).to_string();
                            if collect {
                                Ok((stdout.trim().to_string(), o.status.success()))
                            } else {
                                let msg = if o.status.success() {
                                    "true"
                                } else {
                                    ""
                                };
                                Ok((msg.to_string(), o.status.success()))
                            }
                        }
                        Err(e) => Ok((e.to_string(), false)),
                    }
                },
            )?,
        )?;

        // Regex
        globals.set(
            "vmrRegexpFindString",
            self.lua.create_function(
                |_, (pattern, content): (String, String)| {
                    let re = regex::Regex::new(&pattern);
                    match re {
                        Ok(r) => Ok(r
                            .find(&content)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default()),
                        Err(_) => Ok(String::new()),
                    }
                },
            )?,
        )?;

        // Proxy getter
        globals.set(
            "vmrGetProxy",
            self.lua.create_function(|_, ()| {
                let mut conf = vmr_config::conf::VMRConf::default();
                conf.load();
                match &conf.proxy_uri {
                    Some(uri) if !uri.is_empty() => {
                        if let Some(rest) = uri.strip_prefix("http://") {
                            if let Some((host, port)) = rest.split_once(':') {
                                return Ok((String::from("http"), host.to_string(), port.to_string()));
                            } else {
                                return Ok((String::from("http"), rest.to_string(), String::from("80")));
                            }
                        } else if let Some(rest) = uri.strip_prefix("https://") {
                            if let Some((host, port)) = rest.split_once(':') {
                                return Ok((String::from("https"), host.to_string(), port.to_string()));
                            } else {
                                return Ok((String::from("https"), rest.to_string(), String::from("443")));
                            }
                        }
                        Ok((String::new(), String::new(), String::from("0")))
                    }
                    _ => Ok((String::new(), String::new(), String::from("0"))),
                }
            })?,
        )?;

        // === Version List operations ===
        // vmrNewVersionList() -> table
        // Returns a Lua table (acts as version list: version_name -> array of item tables)
        let new_vl = self.lua.create_function(|lua, ()| {
            let table = lua.create_table()?;
            Ok(table)
        })?;
        globals.set("vmrNewVersionList", new_vl)?;

        // vmrAddItem(vl_table, version_name, item_table) -> vl_table
        // Adds an item table to the version list under the given version name.
        let add_item = self.lua.create_function(
            |lua, (vl, vname, item): (Table, String, Value)| {
                if vname.is_empty() || item.is_nil() {
                    return Ok(vl);
                }
                // Get existing array for this version, or create new one
                let arr: Table = match vl.get::<Table>(vname.as_str()) {
                    Ok(existing) => existing,
                    Err(_) => lua.create_table()?,
                };
                // Append item to array
                let next_idx = arr.raw_len() + 1;
                arr.set(next_idx, item)?;
                vl.set(vname.as_str(), arr)?;
                Ok(vl)
            },
        )?;
        globals.set("vmrAddItem", add_item)?;

        // vmrMergeVersionList(vl1, vl2) -> vl1
        let merge = self.lua.create_function(
            |_lua, (vl1, vl2): (Table, Table)| {
                for pair in vl2.pairs::<String, Value>() {
                    if let Ok((k, v)) = pair {
                        let existing: Result<Table, _> = vl1.get(k.as_str());
                        match existing {
                            Ok(existing_arr) => {
                                // Merge arrays: append items from vl2 to vl1's array
                                if let Some(src_arr) = v.as_table() {
                                    let src_len = src_arr.raw_len();
                                    let dst_len = existing_arr.raw_len();
                                    for i in 1..=src_len {
                                        let val: Value = src_arr.get(i).unwrap_or(Value::Nil);
                                        existing_arr.set(dst_len + i, val).ok();
                                    }
                                    vl1.set(k.as_str(), existing_arr).ok();
                                }
                            }
                            Err(_) => {
                                // New version: copy entire array
                                vl1.set(k.as_str(), v).ok();
                            }
                        }
                    }
                }
                Ok(vl1)
            },
        )?;
        globals.set("vmrMergeVersionList", merge)?;

        // === Installer Config ===
        // vmrNewInstallerConfig() -> table
        globals.set(
            "vmrNewInstallerConfig",
            self.lua.create_function(|lua, ()| {
                let ic = lua.create_table()?;
                ic.set("flag_files", lua.create_table()?)?;
                ic.set("flag_dir_excepted", false)?;
                ic.set("binary_dirs", lua.create_table()?)?;
                ic.set("binary_rename", lua.create_table()?)?;
                ic.set("additional_envs", lua.create_table()?)?;
                Ok(ic)
            })?,
        )?;

        // === Unarchive ===
        globals.set(
            "vmrUnarchive",
            self.lua.create_function(
                |_, (src, dst): (String, String)| {
                    let src_path = std::path::Path::new(&src);
                    let dst_path = std::path::Path::new(&dst);
                    Ok(vmr_utils::archive::extract(src_path, dst_path).is_ok())
                },
            )?,
        )?;

        // === GitHub release downloader (minimal) ===
        globals.set(
            "vmrGetGithubRelease",
            self.lua.create_function(
                |lua,
                 (repo, _token, _proxy, _reverse_proxy): (
                    String,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                )| {
                    if repo.is_empty() {
                        let empty = lua.create_table()?;
                        return Ok(empty);
                    }
                    // Placeholder: fetch GitHub releases and return version list table.
                    // Full implementation would call GitHub API and use Lua callback
                    // functions (tagFilter, versionParser, fileFilter, archParser,
                    // osParser, installerGetter) to build the version list.
                    let url = format!(
                        "https://api.github.com/repos/{}/releases?per_page=100",
                        repo
                    );
                    let client = reqwest::blocking::Client::builder()
                        .user_agent("vmr")
                        .timeout(std::time::Duration::from_secs(60))
                        .build()
                        .unwrap_or_default();

                    let vl = lua.create_table()?;

                    if let Ok(resp) = client.get(&url).send() {
                        if resp.status().is_success() {
                            if let Ok(body) = resp.text() {
                                if let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(&body)
                                {
                                    if let Some(releases) = json.as_array() {
                                        for release in releases {
                                            let tag = release["tag_name"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string();
                                            if tag.is_empty() {
                                                continue;
                                            }
                                            let arr = lua.create_table()?;
                                            if let Some(assets) =
                                                release["assets"].as_array()
                                            {
                                                for (j, asset) in
                                                    assets.iter().enumerate()
                                                {
                                                    let item = lua.create_table()?;
                                                    item.set(
                                                        "url",
                                                        asset["browser_download_url"]
                                                            .as_str()
                                                            .unwrap_or(""),
                                                    )?;
                                                    item.set(
                                                        "arch",
                                                        asset["name"]
                                                            .as_str()
                                                            .unwrap_or(""),
                                                    )?;
                                                    item.set(
                                                        "os",
                                                        asset["content_type"]
                                                            .as_str()
                                                            .unwrap_or(""),
                                                    )?;
                                                    item.set(
                                                        "size",
                                                        asset["size"].as_i64().unwrap_or(0),
                                                    )?;
                                                    item.set(
                                                        "installer",
                                                        UNARCHIVER,
                                                    )?;
                                                    arr.set(j + 1, item)?;
                                                }
                                            }
                                            vl.set(tag, arr)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(vl)
                },
            )?,
        )?;

        Ok(())
    }
}

/// Simple printf-style formatting.
/// Handles positional placeholders (%1, %2, ...) and sequential %s.
fn format_sprintf(pattern: &str, args: &[String]) -> String {
    if args.is_empty() {
        return pattern.to_string();
    }
    let mut result = pattern.to_string();
    // Replace positional placeholders %1, %2, etc.
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("%{}", i + 1);
        result = result.replace(&placeholder, arg);
    }
    // Replace sequential %s placeholders
    let mut arg_iter = args.iter();
    let mut output = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' && chars.peek() == Some(&'s') {
            chars.next(); // consume 's'
            if let Some(arg) = arg_iter.next() {
                output.push_str(arg);
            } else {
                output.push_str("%s");
            }
        } else {
            output.push(c);
        }
    }
    output
}
