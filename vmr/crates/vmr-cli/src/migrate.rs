//! `vmr migrate --asdf | --vfox`: write a `vmr use -f` freeze file from another version
//! manager's local state.
//!
//! Read-only: nothing is installed, activated or removed; the other manager's directories are
//! only read. Each file lists every version the manager has installed. The version it has
//! active globally is written last for each SDK, because `vmr use -f` applies entries
//! sequentially and the last one of an SDK becomes active.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use vmr_core::names::is_valid_name;
use vmr_core::paths;

pub const ASDF_FREEZE_FILE: &str = "migration_from_asdf.txt";
pub const VFOX_FREEZE_FILE: &str = "migration_from_vfox.txt";
pub const SYSTEM_FREEZE_FILE: &str = "migration_from_system.txt";

/// asdf/vfox plugin names whose vmr SDK/plugin name differs.
const ALIASES: &[(&str, &str)] = &[
    ("nodejs", "node"),
    ("golang", "go"),
    ("postgres", "postgresql"),
    ("java", "jdk"),
    ("dotnet-core", "dotnet"),
];

#[derive(Clone, Copy)]
enum Source {
    Asdf,
    Vfox,
    System,
}

impl Source {
    fn label(self) -> &'static str {
        match self {
            Source::Asdf => "asdf",
            Source::Vfox => "vfox",
            Source::System => "system",
        }
    }

    fn freeze_file(self) -> &'static str {
        match self {
            Source::Asdf => ASDF_FREEZE_FILE,
            Source::Vfox => VFOX_FREEZE_FILE,
            Source::System => SYSTEM_FREEZE_FILE,
        }
    }
}

/// Source name -> vmr SDK name; `None` marks an explicit skip.
type Mapping = BTreeMap<String, Option<String>>;

/// Runs `vmr migrate`; the flags may be combined to write several files.
///
/// `map_file` replaces the built-in name mapping (and the system probe defaults) entirely:
/// only names listed there are migrated, so an incomplete file skips the rest.
pub fn run(asdf: bool, vfox: bool, system: bool, map_file: Option<&str>) -> Result<(), String> {
    if !asdf && !vfox && !system {
        return Err("usage: vmr migrate --asdf | --vfox | --system [MAP_FILE]".to_string());
    }
    let (map, map_label) = match map_file {
        Some(file) => (Some(read_mapping(Path::new(file))?), file.to_string()),
        None => (None, String::new()),
    };
    let mut plugins = vmr_lua::Plugins::new();
    let pairs: Vec<(String, String)> = plugins
        .load_all()
        .into_iter()
        .map(|p| (p.plugin_name, p.sdk_name))
        .collect();
    if pairs.is_empty() {
        return Err("no vmr plugins found; run `vmr plugins -u` first".to_string());
    }

    let mut errors = Vec::new();
    for (source, enabled) in [
        (Source::Asdf, asdf),
        (Source::Vfox, vfox),
        (Source::System, system),
    ] {
        if !enabled {
            continue;
        }
        if let Err(e) = migrate(source, &pairs, map.as_ref(), &map_label) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// Parses a mapping file: one `<source name> <vmr sdk>` per line; `#` starts a comment.
///
/// A line with only a source name, or `-` as its target, skips that source explicitly. Later
/// lines win over earlier ones.
fn read_mapping(path: &Path) -> Result<Mapping, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    let mut map = Mapping::new();
    for (index, raw) in content.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let source = tokens.next().expect("non-empty line");
        match (tokens.next(), tokens.next()) {
            (None, _) | (Some("-"), None) => {
                map.insert(source.to_string(), None);
            }
            (Some(target), None) => {
                if !is_valid_name(target) {
                    return Err(format!(
                        "line {}: invalid vmr sdk name `{target}`",
                        index + 1
                    ));
                }
                map.insert(source.to_string(), Some(target.to_string()));
            }
            _ => {
                return Err(format!(
                    "line {}: expected `<source> <vmr-sdk>` or `<source> -`, got `{line}`",
                    index + 1
                ));
            }
        }
    }
    if map.is_empty() {
        return Err(format!("{} has no mapping entries", path.display()));
    }
    Ok(map)
}

/// Exact match first, then a case-insensitive one (vfox lowercases install directories).
fn lookup_mapping<'a>(map: &'a Mapping, name: &str) -> Option<Option<&'a str>> {
    map.get(name)
        .or_else(|| {
            map.iter()
                .find(|(source, _)| source.eq_ignore_ascii_case(name))
                .map(|(_, target)| target)
        })
        .map(|target| target.as_deref())
}

/// State read from one manager: installed versions, its global pins and report notes.
struct State {
    /// What the state was read from (printed in the report).
    origin: String,
    versions: BTreeMap<String, BTreeSet<String>>,
    global: BTreeMap<String, Vec<String>>,
    notes: Vec<String>,
}

fn migrate(
    source: Source,
    pairs: &[(String, String)],
    map: Option<&Mapping>,
    map_label: &str,
) -> Result<(), String> {
    let state = match source {
        Source::Asdf => asdf_state()?,
        Source::Vfox => vfox_state()?,
        Source::System => system_state(map, map_label)?,
    };
    // The system source already applied the map to its command names; its keys are SDK names.
    let use_map = !matches!(source, Source::System);
    let plan = build_plan(
        &state.versions,
        &state.global,
        pairs,
        map,
        map_label,
        use_map,
    );
    let mut notes = state.notes;
    notes.extend(plan.notes);

    if plan.entries.is_empty() {
        for note in &notes {
            eprintln!("note: {note}");
        }
        return Err(format!(
            "no {} versions map to a vmr plugin; nothing written",
            source.label()
        ));
    }

    let path = PathBuf::from(source.freeze_file());
    fs::write(&path, render(&plan.entries, source))
        .map_err(|e| format!("write {} failed: {e}", path.display()))?;

    let sdk_count = plan
        .entries
        .iter()
        .filter_map(|e| e.split_once('@').map(|(sdk, _)| sdk))
        .collect::<BTreeSet<_>>()
        .len();
    println!("{} data: {}", source.label(), state.origin);
    println!(
        "wrote {} version(s) for {} SDK(s) to {}",
        plan.entries.len(),
        sdk_count,
        path.display()
    );
    println!(
        "no installation performed; replay with: vmr use -f {}",
        path.display()
    );
    for note in &notes {
        eprintln!("note: {note}");
    }
    Ok(())
}

// ---------------------------------------------------------------- asdf

/// `ASDF_DATA_DIR`, else `~/.asdf` when it holds asdf state, else `ASDF_DIR`.
fn asdf_data_dir() -> Option<PathBuf> {
    if let Some(dir) = non_empty_env("ASDF_DATA_DIR") {
        return Some(PathBuf::from(dir));
    }
    let default = paths::home_dir().join(".asdf");
    if default.join("installs").is_dir() || default.join("plugins").is_dir() {
        return Some(default);
    }
    non_empty_env("ASDF_DIR").map(PathBuf::from)
}

fn asdf_state() -> Result<State, String> {
    let data_dir = asdf_data_dir().ok_or_else(|| {
        "asdf data directory not found; set ASDF_DATA_DIR or install asdf under ~/.asdf".to_string()
    })?;
    let (versions, mut notes) = read_asdf_installs(&data_dir)
        .map_err(|e| format!("read {} failed: {e}", data_dir.display()))?;
    if versions.is_empty() {
        return Err(format!(
            "no asdf-installed versions found under {}",
            data_dir.join("installs").display()
        ));
    }
    let (global, mut global_notes) = read_asdf_global_versions();
    notes.append(&mut global_notes);
    Ok(State {
        origin: data_dir.to_string_lossy().into_owned(),
        versions,
        global,
        notes,
    })
}

/// Reads `<data>/installs/<plugin>/<version>/`.
///
/// Skips `ref-*` (asdf's on-disk form of `ref:<branch>`, which vmr cannot express), names that
/// are not safe path components, and versions whose `<data>/install-locks` marker exists (an
/// interrupted install, which asdf itself does not count as installed).
fn read_asdf_installs(
    data_dir: &Path,
) -> std::io::Result<(BTreeMap<String, BTreeSet<String>>, Vec<String>)> {
    let root = data_dir.join("installs");
    let mut versions = BTreeMap::new();
    let mut notes = Vec::new();
    for plugin in dirs_under(&root)? {
        let locks = data_dir.join("install-locks").join(&plugin);
        let mut found = BTreeSet::new();
        for version in dirs_under(&root.join(&plugin))? {
            if version.starts_with("ref-") {
                notes.push(format!("{plugin}: skipped ref: version `{version}`"));
                continue;
            }
            if !is_valid_name(&version) {
                notes.push(format!(
                    "{plugin}: skipped version `{version}` (invalid name)"
                ));
                continue;
            }
            if locks.join(&version).exists() {
                continue;
            }
            found.insert(version);
        }
        if !found.is_empty() {
            versions.insert(plugin, found);
        }
    }
    Ok((versions, notes))
}

/// asdf's tool-versions filename: `ASDF_TOOL_VERSIONS_FILENAME` (new name), then the legacy
/// `ASDF_DEFAULT_TOOL_VERSIONS_FILENAME`, then `.tool-versions`.
fn asdf_tool_versions_filename() -> String {
    non_empty_env("ASDF_TOOL_VERSIONS_FILENAME")
        .or_else(|| non_empty_env("ASDF_DEFAULT_TOOL_VERSIONS_FILENAME"))
        .unwrap_or_else(|| ".tool-versions".to_string())
}

/// Global versions: `$HOME/<tool-versions filename>` (asdf's global file).
fn read_asdf_global_versions() -> (BTreeMap<String, Vec<String>>, Vec<String>) {
    let path = paths::home_dir().join(asdf_tool_versions_filename());
    let candidates = fs::read_to_string(path)
        .map(|content| parse_tool_versions(&content))
        .unwrap_or_default();
    (candidates, Vec::new())
}

/// Parses `.tool-versions`: `#` starts a comment anywhere; the first token is the tool and the
/// rest are versions. The first line for a tool wins (asdf's lookup stops at the first match).
fn parse_tool_versions(content: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for raw in content.lines() {
        let line = raw.split('#').next().unwrap_or_default();
        let mut tokens = line.split_whitespace();
        let Some(name) = tokens.next() else {
            continue;
        };
        let versions: Vec<String> = tokens.map(str::to_string).collect();
        if versions.is_empty() {
            continue;
        }
        out.entry(name.to_string()).or_insert(versions);
    }
    out
}

// ---------------------------------------------------------------- vfox

/// vfox's user directory: `~/.version-fox` when it exists (legacy), else `~/.vfox`.
///
/// `VFOX_HOME` does not move it: it replaces only the shared root (installs/plugins/config).
fn vfox_user_home() -> PathBuf {
    let legacy = paths::home_dir().join(".version-fox");
    if legacy.is_dir() {
        return legacy;
    }
    paths::home_dir().join(".vfox")
}

fn vfox_state() -> Result<State, String> {
    let user_home = vfox_user_home();
    let root = non_empty_env("VFOX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home.clone());

    let override_path = storage_sdk_path(&[
        // The user config overrides the shared one (`VFOX_HOME/config.yaml`).
        root.join("config.yaml"),
        user_home.join("config.yaml"),
    ]);
    let installs = override_path.clone().unwrap_or_else(|| root.join("cache"));
    let (versions, mut notes) = read_vfox_installs(&installs)
        .map_err(|e| format!("read {} failed: {e}", installs.display()))?;
    if versions.is_empty() {
        return Err(format!(
            "no vfox-installed versions found under {}",
            installs.display()
        ));
    }
    let (global, mut global_notes) = read_vfox_global_versions(&user_home);
    notes.append(&mut global_notes);
    let mut origin = installs.to_string_lossy().into_owned();
    if override_path.is_some() {
        origin.push_str(" (storage.sdkPath)");
    }
    Ok(State {
        origin,
        versions,
        global,
        notes,
    })
}

/// Reads `<cache>/<sdk>/v-<version>/`.
///
/// Mirrors vfox's own `InstalledList`/`CheckRuntimeExist`: only `v-*` directories, no
/// `.v-<version>.installing` marker, and a runtime payload whose name ends with
/// `-<version>` (vfox places the main runtime at `v-<version>/<name>-<version>`; following the
/// link keeps a broken link out).
fn read_vfox_installs(
    installs: &Path,
) -> std::io::Result<(BTreeMap<String, BTreeSet<String>>, Vec<String>)> {
    let mut versions = BTreeMap::new();
    let mut notes = Vec::new();
    for sdk in dirs_under(installs)? {
        let sdk_dir = installs.join(&sdk);
        let mut found = BTreeSet::new();
        for entry in dirs_under(&sdk_dir)? {
            let Some(version) = entry.strip_prefix("v-") else {
                continue;
            };
            if version.is_empty() {
                continue;
            }
            if !is_valid_name(version) {
                notes.push(format!("{sdk}: skipped version `{version}` (invalid name)"));
                continue;
            }
            if sdk_dir.join(format!(".v-{version}.installing")).exists() {
                continue;
            }
            if !has_vfox_runtime(&sdk_dir.join(&entry), version) {
                notes.push(format!(
                    "{sdk}: skipped `{version}` (no runtime payload; interrupted or broken link)"
                ));
                continue;
            }
            found.insert(version.to_string());
        }
        if !found.is_empty() {
            versions.insert(sdk, found);
        }
    }
    Ok((versions, notes))
}

/// Whether `version_dir` holds the main runtime (`<name>-<version>`), following symlinks.
fn has_vfox_runtime(version_dir: &Path, version: &str) -> bool {
    let suffix = format!("-{version}");
    fs::read_dir(version_dir)
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry.file_name().to_string_lossy().ends_with(&suffix) && entry.path().is_dir()
            })
        })
        .unwrap_or(false)
}

/// `storage.sdkPath` from the config files, later files winning (`~/.vfox/config.yaml`).
///
/// vfox writes the config as block YAML with `sdkPath` only under `storage`, so a line scan is
/// enough; no YAML parser is pulled in for this single key.
fn storage_sdk_path(configs: &[PathBuf]) -> Option<PathBuf> {
    let mut out = None;
    for path in configs {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            if key.trim() != "sdkPath" {
                continue;
            }
            let value = value.split(" #").next().unwrap_or(value).trim();
            let value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                out = Some(PathBuf::from(value));
            }
        }
    }
    out
}

/// Global versions: `<user home>/.vfox.toml`, then `vfox.toml`, then `.tool-versions`.
/// (vfox prefers the TOML files and migrates `.tool-versions` silently.)
fn read_vfox_global_versions(user_home: &Path) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
    for name in [".vfox.toml", "vfox.toml"] {
        let path = user_home.join(name);
        if !path.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            return (BTreeMap::new(), Vec::new());
        };
        return parse_vfox_toml(&content);
    }
    let path = user_home.join(".tool-versions");
    let candidates = fs::read_to_string(path)
        .map(|content| parse_tool_versions(&content))
        .unwrap_or_default();
    (candidates, Vec::new())
}

/// `[tools]` from a vfox TOML file: `name = "version"` or
/// `name = { version = "1.2.3", vendor = "..." }`. Attributes have no vmr equivalent.
fn parse_vfox_toml(content: &str) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
    let value: toml::Value = match toml::from_str(content) {
        Ok(value) => value,
        Err(e) => {
            return (
                BTreeMap::new(),
                vec![format!("failed to parse global .vfox.toml: {e}")],
            );
        }
    };
    let mut out = BTreeMap::new();
    let mut notes = Vec::new();
    let Some(tools) = value.get("tools").and_then(toml::Value::as_table) else {
        return (out, notes);
    };
    for (name, value) in tools {
        match value {
            toml::Value::String(version) if !version.is_empty() => {
                out.insert(name.clone(), vec![version.clone()]);
            }
            toml::Value::Table(table) => match table.get("version").and_then(toml::Value::as_str) {
                Some(version) if !version.is_empty() => {
                    let attrs: Vec<&str> = table
                        .keys()
                        .filter(|key| *key != "version")
                        .map(String::as_str)
                        .collect();
                    if !attrs.is_empty() {
                        notes.push(format!("{name}: attributes ignored ({})", attrs.join(", ")));
                    }
                    out.insert(name.clone(), vec![version.to_string()]);
                }
                _ => notes.push(format!("{name}: no version in global .vfox.toml")),
            },
            _ => notes.push(format!("{name}: unsupported value in global .vfox.toml")),
        }
    }
    (out, notes)
}

// ---------------------------------------------------------------- system

/// One system command to probe. Only SDKs the vmr plugin bundle can install are listed.
struct Probe {
    /// vmr SDK name.
    sdk: &'static str,
    binary: &'static str,
    args: &'static [&'static str],
    /// Regex whose first capture group is the version.
    pattern: &'static str,
}

/// Common system-installed SDKs (package managers, official installers). The first match on
/// `PATH` wins; a later entry for the same SDK is not probed.
const PROBES: &[Probe] = &[
    Probe {
        sdk: "node",
        binary: "node",
        args: &["--version"],
        pattern: r"v?(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "go",
        binary: "go",
        args: &["version"],
        pattern: r"go(\d+\.\d+(?:\.\d+)?)",
    },
    Probe {
        sdk: "python",
        binary: "python3",
        args: &["--version"],
        pattern: r"Python (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "python",
        binary: "python",
        args: &["--version"],
        pattern: r"Python (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "ruby",
        binary: "ruby",
        args: &["--version"],
        pattern: r"ruby (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "jdk",
        binary: "java",
        args: &["-version"],
        pattern: r#"version "([0-9][^"]*)""#,
    },
    Probe {
        sdk: "rust",
        binary: "rustc",
        args: &["--version"],
        pattern: r"rustc (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "gcc",
        binary: "gcc",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "gxx",
        binary: "g++",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "clang",
        binary: "clang",
        args: &["--version"],
        pattern: r"clang version (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "deno",
        binary: "deno",
        args: &["--version"],
        pattern: r"deno (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "bun",
        binary: "bun",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "zig",
        binary: "zig",
        args: &["version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "dotnet",
        binary: "dotnet",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "php",
        binary: "php",
        args: &["--version"],
        pattern: r"PHP (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "lua",
        binary: "lua",
        args: &["-v"],
        pattern: r"Lua (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "julia",
        binary: "julia",
        args: &["--version"],
        pattern: r"julia version (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "kotlin",
        binary: "kotlinc",
        args: &["-version"],
        pattern: r"kotlinc-jvm (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "gradle",
        binary: "gradle",
        args: &["--version"],
        pattern: r"Gradle (\d+\.\d+(?:\.\d+)?)",
    },
    Probe {
        sdk: "maven",
        binary: "mvn",
        args: &["-v"],
        pattern: r"Apache Maven (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "cmake",
        binary: "cmake",
        args: &["--version"],
        pattern: r"cmake version (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "ninja",
        binary: "ninja",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "bazel",
        binary: "bazel",
        args: &["--version"],
        pattern: r"bazel (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "kubectl",
        binary: "kubectl",
        args: &["version", "--client"],
        pattern: r"(?:Client Version: |gitVersion: )v?(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "protoc",
        binary: "protoc",
        args: &["--version"],
        pattern: r"libprotoc (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "neovim",
        binary: "nvim",
        args: &["--version"],
        pattern: r"NVIM v(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "ripgrep",
        binary: "rg",
        args: &["--version"],
        pattern: r"ripgrep (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "fzf",
        binary: "fzf",
        args: &["--version"],
        pattern: r"(\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "fd",
        binary: "fd",
        args: &["--version"],
        pattern: r"fd (\d+\.\d+\.\d+)",
    },
    Probe {
        sdk: "upx",
        binary: "upx",
        args: &["--version"],
        pattern: r"upx (\d+\.\d+\.\d+)",
    },
];

fn system_state(map: Option<&Mapping>, map_label: &str) -> Result<State, String> {
    let paths: Vec<PathBuf> = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default();
    let roots = managed_roots();
    let mut versions = BTreeMap::new();
    let mut notes = Vec::new();

    for probe in PROBES {
        // Only commands that are actually installed are considered: a missing one is not worth
        // a report, and the mapping is applied after this check so it cannot create noise.
        let Some((path, managed)) = find_in_paths(probe.binary, &paths, &roots) else {
            continue;
        };
        if let Some(label) = managed {
            notes.push(format!(
                "{}: {} is managed by {label}; skipped",
                probe.binary,
                path.display()
            ));
            continue;
        }
        // With a mapping file the command name is the key and the file is authoritative.
        let sdk = match map {
            Some(map) => match lookup_mapping(map, probe.binary) {
                None => {
                    notes.push(format!("{}: not in {map_label}; skipped", probe.binary));
                    continue;
                }
                Some(None) => continue,
                Some(Some(target)) => target.to_string(),
            },
            None => probe.sdk.to_string(),
        };
        if versions.contains_key(&sdk) {
            continue;
        }
        let output = run_probe(&path, probe.args);
        let Some(version) = extract_version(probe.pattern, &output) else {
            notes.push(format!(
                "{}: could not parse a version from `{} {}`",
                probe.binary,
                probe.binary,
                probe.args.join(" ")
            ));
            continue;
        };
        if !is_valid_name(&version) {
            notes.push(format!("{sdk}: unsupported version `{version}`"));
            continue;
        }
        versions.insert(sdk, BTreeSet::from([version]));
    }

    if versions.is_empty() {
        return Err("no system-installed SDK commands matched; nothing written".to_string());
    }
    Ok(State {
        origin: "PATH".to_string(),
        versions,
        global: BTreeMap::new(),
        notes,
    })
}

/// Runs the probe and returns stdout plus stderr (java and kotlinc print their version to
/// stderr).
fn run_probe(binary: &Path, args: &[&str]) -> String {
    match std::process::Command::new(binary).args(args).output() {
        Ok(output) => {
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            text.push('\n');
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            text
        }
        Err(_) => String::new(),
    }
}

fn extract_version(pattern: &str, output: &str) -> Option<String> {
    let regex = Regex::new(pattern).ok()?;
    regex
        .captures(output)?
        .get(1)
        .map(|matched| matched.as_str().to_string())
}

/// First executable on `PATH` and, when it lives under a known version manager, that manager.
fn find_in_paths(
    binary: &str,
    paths: &[PathBuf],
    roots: &[(&'static str, PathBuf)],
) -> Option<(PathBuf, Option<&'static str>)> {
    for dir in paths {
        for candidate in candidate_names(dir, binary) {
            if !is_executable_file(&candidate) {
                continue;
            }
            let managed = roots
                .iter()
                .find(|(_, root)| is_under(&candidate, root))
                .map(|(label, _)| *label);
            return Some((candidate, managed));
        }
    }
    None
}

#[allow(unused_mut)]
fn candidate_names(dir: &Path, binary: &str) -> Vec<PathBuf> {
    let mut out = vec![dir.join(binary)];
    #[cfg(windows)]
    if !binary.contains('.') {
        out.push(dir.join(format!("{binary}.exe")));
    }
    out
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    match fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

fn is_under(path: &Path, root: &Path) -> bool {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    path.starts_with(root)
}

/// Directories owned by version managers (including vmr itself), so a binary found there is not
/// a system install.
fn managed_roots() -> Vec<(&'static str, PathBuf)> {
    let home = paths::home_dir();
    let mut roots = vec![
        ("vmr", home.join(".vmr")),
        ("asdf", home.join(".asdf")),
        ("vfox", home.join(".vfox")),
        ("vfox", home.join(".version-fox")),
        ("nvm", home.join(".nvm")),
        ("fnm", home.join(".fnm")),
        ("fnm", home.join(".local").join("share").join("fnm")),
        ("volta", home.join(".volta")),
        ("pyenv", home.join(".pyenv")),
        ("rbenv", home.join(".rbenv")),
        ("sdkman", home.join(".sdkman")),
        ("mise", home.join(".mise")),
        ("mise", home.join(".local").join("share").join("mise")),
        ("rtx", home.join(".local").join("share").join("rtx")),
        ("conda", home.join("miniconda3")),
        ("conda", home.join("anaconda3")),
        ("conda", home.join("miniforge3")),
    ];
    if let Some(dir) = non_empty_env("VMR_SDK_INSTALLATION_DIR") {
        roots.push(("vmr", PathBuf::from(dir)));
    }
    if let Some(dir) = non_empty_env("ASDF_DATA_DIR") {
        roots.push(("asdf", PathBuf::from(dir)));
    }
    if let Some(dir) = non_empty_env("VFOX_HOME") {
        roots.push(("vfox", PathBuf::from(dir)));
    }
    roots
}

/// The version the manager resolves: the first candidate that is actually installed. `system`,
/// `ref:` and `path:` never match an install directory, so they yield no active version.
fn pick_current(candidates: Option<&Vec<String>>, installed: &BTreeSet<String>) -> Option<String> {
    candidates?
        .iter()
        .find(|version| installed.contains(*version))
        .cloned()
}

/// Global candidates for a tool, matching case-insensitively (`vfox` lowercases its install
/// directories while the version file keeps the plugin's spelling).
fn global_candidates<'a>(
    global: &'a BTreeMap<String, Vec<String>>,
    name: &str,
) -> Option<&'a Vec<String>> {
    global.get(name).or_else(|| {
        global
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, versions)| versions)
    })
}

/// Built-in name matching: exact plugin/sdk match, the alias table, then case-insensitive.
fn builtin_sdk(pairs: &[(String, String)], name: &str) -> Option<String> {
    let find = |name: &str| {
        pairs
            .iter()
            .find(|(plugin, sdk)| plugin == name || sdk == name)
            .map(|(_, sdk)| sdk.clone())
    };
    if let Some(sdk) = find(name) {
        return Some(sdk);
    }
    if let Some((_, target)) = ALIASES.iter().find(|(alias, _)| *alias == name) {
        if let Some(sdk) = find(target) {
            return Some(sdk);
        }
    }
    pairs
        .iter()
        .find(|(plugin, sdk)| plugin.eq_ignore_ascii_case(name) || sdk.eq_ignore_ascii_case(name))
        .map(|(_, sdk)| sdk.clone())
}

enum Resolution {
    Sdk(String),
    /// The mapping file marks this name as skipped.
    Skip,
    /// No mapping and no plugin: report the note and drop the versions.
    Unmapped(String),
}

/// Resolves a source name through the mapping file when one is given (it is authoritative),
/// otherwise through the built-in matching.
fn resolve_name(
    pairs: &[(String, String)],
    name: &str,
    map: Option<&Mapping>,
    map_label: &str,
    use_map: bool,
) -> Resolution {
    if use_map {
        if let Some(map) = map {
            return match lookup_mapping(map, name) {
                None => Resolution::Unmapped(format!("{name}: not in {map_label}; skipped")),
                Some(None) => Resolution::Skip,
                Some(Some(target)) => match builtin_sdk(pairs, target) {
                    Some(sdk) => Resolution::Sdk(sdk),
                    None => Resolution::Unmapped(format!(
                        "{name}: mapped to `{target}`, which is not a vmr plugin; skipped"
                    )),
                },
            };
        }
    }
    match builtin_sdk(pairs, name) {
        Some(sdk) => Resolution::Sdk(sdk),
        None => Resolution::Unmapped(format!(
            "{name}: no vmr plugin; its versions were not written"
        )),
    }
}

struct Plan {
    /// Freeze lines (`sdk@version`), current version last per SDK.
    entries: Vec<String>,
    notes: Vec<String>,
}

/// Builds the freeze entries. Order inside an SDK: `vmr freeze` order (descending), with the
/// active version moved last so `vmr use -f` finishes on it.
fn build_plan(
    installs: &BTreeMap<String, BTreeSet<String>>,
    global: &BTreeMap<String, Vec<String>>,
    pairs: &[(String, String)],
    map: Option<&Mapping>,
    map_label: &str,
    use_map: bool,
) -> Plan {
    // SDK -> (versions, active version). Two plugin names (e.g. `go` and `golang`) can map to
    // the same SDK; their versions are merged and the first active version is kept.
    let mut by_sdk: BTreeMap<String, (BTreeSet<String>, Option<String>)> = BTreeMap::new();
    let mut notes = Vec::new();

    for (name, versions) in installs {
        let sdk = match resolve_name(pairs, name, map, map_label, use_map) {
            Resolution::Sdk(sdk) => sdk,
            Resolution::Skip => continue,
            Resolution::Unmapped(note) => {
                notes.push(note);
                continue;
            }
        };
        let current = pick_current(global_candidates(global, name), versions);
        let slot = by_sdk.entry(sdk.clone()).or_default();
        slot.0.extend(versions.iter().cloned());
        match slot.1.take() {
            Some(existing) => {
                if current.as_ref().is_some_and(|cur| *cur != existing) {
                    notes.push(format!(
                        "{sdk}: plugins disagree on the active version; keeping `{existing}`"
                    ));
                }
                slot.1 = Some(existing);
            }
            None => slot.1 = current,
        }
    }

    let mut entries = Vec::new();
    for (sdk, (versions, current)) in by_sdk {
        let mut ordered: Vec<String> = versions.into_iter().collect();
        vmr_utils::semver::sort_versions(&mut ordered);
        if let Some(current) = current {
            if let Some(pos) = ordered.iter().position(|v| v == &current) {
                ordered.remove(pos);
            }
            ordered.push(current);
        }
        for version in ordered {
            entries.push(format!("{sdk}@{version}"));
        }
    }
    Plan { entries, notes }
}

fn render(entries: &[String], source: Source) -> String {
    let trailing = match source {
        Source::System => "# collected from the SDK commands found on PATH\n".to_string(),
        _ => format!(
            "# the last entry of each SDK is the version {} had active\n",
            source.label()
        ),
    };
    let mut text = format!(
        "# vmr migrate --{} {}\n# replay with: vmr use -f {}\n{}",
        source.label(),
        chrono::Local::now().format("%Y-%m-%d"),
        source.freeze_file(),
        trailing,
    );
    for entry in entries {
        text.push_str(entry);
        text.push('\n');
    }
    text
}

// ---------------------------------------------------------------- helpers

fn non_empty_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Sorted directory names under `path`; a missing directory is empty, not an error.
fn dirs_under(path: &Path) -> std::io::Result<Vec<String>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .collect()
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "vmr-migrate-{}-{tag}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn parses_comments_and_keeps_the_first_line_per_tool() {
        let data = "ruby 3.2.0 # pinned\ngo 1.22.0 1.21.0\n\npython 3.12.1\nruby 3.3.0\n";
        let map = parse_tool_versions(data);
        assert_eq!(map["ruby"], vec!["3.2.0"]);
        assert_eq!(map["go"], vec!["1.22.0", "1.21.0"]);
        assert_eq!(map["python"], vec!["3.12.1"]);
    }

    #[test]
    fn picks_the_first_installed_candidate() {
        let installed = BTreeSet::from(["2.0.0".to_string()]);
        let candidates = vec![
            "system".to_string(),
            "1.0.0".to_string(),
            "2.0.0".to_string(),
        ];
        assert_eq!(
            pick_current(Some(&candidates), &installed).as_deref(),
            Some("2.0.0")
        );
        assert_eq!(pick_current(None, &installed), None);
    }

    #[test]
    fn maps_aliases_and_orders_the_active_version_last() {
        let installs = BTreeMap::from([(
            "nodejs".to_string(),
            BTreeSet::from(["20.9.0".to_string(), "20.11.0".to_string()]),
        )]);
        let global = BTreeMap::from([("nodejs".to_string(), vec!["20.11.0".to_string()])]);
        let plan = build_plan(
            &installs,
            &global,
            &pairs(&[("node", "node")]),
            None,
            "",
            true,
        );
        assert_eq!(plan.entries, vec!["node@20.9.0", "node@20.11.0"]);
        assert!(plan.notes.is_empty());
    }

    #[test]
    fn reports_plugins_without_a_vmr_equivalent() {
        let installs = BTreeMap::from([(
            "terraform".to_string(),
            BTreeSet::from(["1.7.0".to_string()]),
        )]);
        let plan = build_plan(
            &installs,
            &BTreeMap::new(),
            &pairs(&[("go", "go")]),
            None,
            "",
            true,
        );
        assert!(plan.entries.is_empty());
        assert!(plan.notes[0].contains("terraform"));
        assert!(plan.notes[0].contains("no vmr plugin"));
    }

    #[test]
    fn merges_two_plugin_names_that_map_to_one_sdk() {
        let installs = BTreeMap::from([
            ("go".to_string(), BTreeSet::from(["1.22.0".to_string()])),
            ("golang".to_string(), BTreeSet::from(["1.21.0".to_string()])),
        ]);
        let global = BTreeMap::from([("golang".to_string(), vec!["1.21.0".to_string()])]);
        let plan = build_plan(&installs, &global, &pairs(&[("go", "go")]), None, "", true);
        assert_eq!(plan.entries, vec!["go@1.22.0", "go@1.21.0"]);
    }

    #[test]
    fn read_asdf_installs_skips_ref_and_incomplete_versions() {
        let root = temp_dir("asdf");
        fs::create_dir_all(root.join("installs/nodejs/20.11.0")).unwrap();
        fs::create_dir_all(root.join("installs/nodejs/ref-master")).unwrap();
        fs::create_dir_all(root.join("installs/nodejs/20.10.0")).unwrap();
        fs::create_dir_all(root.join("install-locks/nodejs/20.10.0")).unwrap();
        fs::write(root.join("installs/nodejs/README"), "not a version").unwrap();

        let (installs, notes) = read_asdf_installs(&root).unwrap();
        assert_eq!(installs["nodejs"], BTreeSet::from(["20.11.0".to_string()]));
        assert!(notes.iter().any(|n| n.contains("ref-")));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn parses_vfox_toml_versions_and_reports_attributes() {
        let data =
            "[tools]\nnodejs = \"21.5.1\"\njava = { version = \"21\", vendor = \"openjdk\" }\n";
        let (tools, notes) = parse_vfox_toml(data);
        assert_eq!(tools["nodejs"], vec!["21.5.1"]);
        assert_eq!(tools["java"], vec!["21"]);
        assert!(notes.iter().any(|n| n.contains("vendor")));
    }

    #[test]
    fn read_vfox_installs_matches_vfox_layout() {
        let root = temp_dir("vfox");
        let cache = root.join("cache");
        // Valid main runtime.
        fs::create_dir_all(cache.join("nodejs/v-21.5.1/nodejs-21.5.1")).unwrap();
        // Interrupted install: payload exists but the marker does too.
        fs::create_dir_all(cache.join("nodejs/v-20.0.0/nodejs-20.0.0")).unwrap();
        fs::write(cache.join("nodejs/.v-20.0.0.installing"), "").unwrap();
        // Version directory without its runtime payload.
        fs::create_dir_all(cache.join("nodejs/v-19.0.0")).unwrap();
        // Additional runtime is not a version.
        fs::create_dir_all(cache.join("nodejs/v-21.5.1/add-openjdk-17")).unwrap();

        let (installs, notes) = read_vfox_installs(&cache).unwrap();
        assert_eq!(installs["nodejs"], BTreeSet::from(["21.5.1".to_string()]));
        assert!(
            notes
                .iter()
                .any(|n| n.contains("20.0.0") || n.contains("19.0.0"))
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scans_storage_sdk_path_from_config_files() {
        let root = temp_dir("config");
        let shared = root.join("shared.yaml");
        let user = root.join("user.yaml");
        fs::write(&shared, "storage:\n  sdkPath: /opt/sdks\n").unwrap();
        fs::write(&user, "# nothing here\nproxy:\n  enable: false\n").unwrap();
        // The user config overrides the shared one only when it sets the key.
        assert_eq!(
            storage_sdk_path(&[shared.clone(), user.clone()]),
            Some(PathBuf::from("/opt/sdks"))
        );
        fs::write(&user, "storage:\n  sdkPath: \"/data/sdks\" # comment\n").unwrap();
        assert_eq!(
            storage_sdk_path(&[shared, user]),
            Some(PathBuf::from("/data/sdks"))
        );

        let _ = fs::remove_dir_all(&root);
    }

    fn probe(binary: &str) -> &'static Probe {
        PROBES
            .iter()
            .find(|probe| probe.binary == binary)
            .expect("probe exists")
    }

    #[test]
    fn probe_patterns_extract_common_versions() {
        for probe in PROBES {
            Regex::new(probe.pattern).expect("valid probe pattern");
        }
        let cases = [
            ("node", "v24.0.1\n", "24.0.1"),
            ("go", "go version go1.22.3 linux/amd64\n", "1.22.3"),
            ("python3", "Python 3.12.1\n", "3.12.1"),
            (
                "ruby",
                "ruby 3.3.0p0 (2023-12-25 revision 10590)\n",
                "3.3.0",
            ),
            ("java", "openjdk version \"21.0.2\" 2024-01-16\n", "21.0.2"),
            ("rustc", "rustc 1.75.0 (82e1608df 2023-12-21)\n", "1.75.0"),
            ("clang", "clang version 17.0.6\n", "17.0.6"),
            ("bun", "1.0.20\n", "1.0.20"),
            ("deno", "deno 1.40.0\n", "1.40.0"),
            ("gradle", "Gradle 8.5\n", "8.5"),
            ("cmake", "cmake version 3.28.1\n", "3.28.1"),
            ("kubectl", "Client Version: v1.29.0\n", "1.29.0"),
            ("nvim", "NVIM v0.9.5\n", "0.9.5"),
        ];
        for (binary, output, expected) in cases {
            assert_eq!(
                extract_version(probe(binary).pattern, output).as_deref(),
                Some(expected),
                "{binary}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn find_in_paths_prefers_the_first_executable_and_labels_managers() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("path");
        let system = root.join("system");
        let managed = root.join(".nvm/versions/node/v20/bin");
        fs::create_dir_all(&system).unwrap();
        fs::create_dir_all(&managed).unwrap();
        for dir in [&system, &managed] {
            let file = dir.join("node");
            fs::write(&file, "#!/bin/sh\n").unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let roots = vec![("nvm", root.join(".nvm"))];

        assert_eq!(
            find_in_paths("node", &[system.clone(), managed.clone()], &roots),
            Some((system.join("node"), None))
        );
        assert_eq!(
            find_in_paths("node", &[managed.clone(), system], &roots),
            Some((managed.join("node"), Some("nvm")))
        );

        // A non-executable file is not a command.
        let plain = root.join("plain");
        fs::create_dir_all(&plain).unwrap();
        fs::write(plain.join("node"), "").unwrap();
        assert_eq!(find_in_paths("node", &[plain], &roots), None);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reads_mapping_file_with_skips_and_comments() {
        let root = temp_dir("map");
        let path = root.join("map.txt");
        fs::write(
            &path,
            "# comment\nnodejs node\nruby -\ngo\n\npython python  # trailing\n",
        )
        .unwrap();
        let map = read_mapping(&path).unwrap();
        assert_eq!(map["nodejs"], Some("node".to_string()));
        assert_eq!(map["ruby"], None);
        assert_eq!(map["go"], None);
        assert_eq!(map["python"], Some("python".to_string()));
        assert_eq!(lookup_mapping(&map, "NODEJS"), Some(Some("node")));
        assert_eq!(lookup_mapping(&map, "missing"), None);

        fs::write(&path, "node invalid/name\n").unwrap();
        assert!(read_mapping(&path).is_err());
        fs::write(&path, "a b c\n").unwrap();
        assert!(read_mapping(&path).is_err());
        fs::write(&path, "# only comments\n").unwrap();
        assert!(read_mapping(&path).is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn mapping_file_is_authoritative() {
        let installs = BTreeMap::from([
            (
                "nodejs".to_string(),
                BTreeSet::from(["20.11.0".to_string()]),
            ),
            ("ruby".to_string(), BTreeSet::from(["3.3.0".to_string()])),
            (
                "terraform".to_string(),
                BTreeSet::from(["1.7.0".to_string()]),
            ),
        ]);
        let map: Mapping = BTreeMap::from([
            ("nodejs".to_string(), Some("go".to_string())),
            ("ruby".to_string(), None),
        ]);
        let plan = build_plan(
            &installs,
            &BTreeMap::new(),
            &pairs(&[("go", "go"), ("node", "node")]),
            Some(&map),
            "map.txt",
            true,
        );
        assert_eq!(plan.entries, vec!["go@20.11.0"]);
        assert!(
            plan.notes
                .iter()
                .any(|note| note.contains("terraform") && note.contains("not in map.txt"))
        );
        assert!(!plan.notes.iter().any(|note| note.contains("ruby")));
    }
}
