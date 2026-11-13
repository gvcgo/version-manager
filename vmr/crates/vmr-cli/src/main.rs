//! vmr CLI (clap; commands/aliases mirror the Go `cmd/vmr` table; no TUI).

mod doctor;
mod mcp;
mod table;

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser, Subcommand};

use vmr_core::envs;
use vmr_core::names::{check_name, is_valid_name};
use vmr_core::paths;
use vmr_installer::{common, finder, installer as ins, locker, pixi_env};
use vmr_lua::types::{Item, installer_kind};
use vmr_lua::{Plugin, Plugins};

#[derive(Parser)]
#[command(
    name = "vmr",
    version = version_str(),
    about = "A cross-platform SDK/version manager (Rust rewrite).",
    disable_help_subcommand = false
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show version info.
    #[command(visible_alias = "v")]
    Version,
    /// Print the shell integration script (`eval "$(vmr init zsh)"`).
    Init { shell: Option<String> },
    /// Print a Lua plugin template (`github`, `html`, `json`, `pixi`).
    Template {
        #[arg(help = "template kind: github | html | json | pixi")]
        kind: Option<String>,
        #[arg(
            short,
            long,
            help = "write <kind>.lua into this directory instead of stdout"
        )]
        dir: Option<String>,
    },
    /// cd hook: install missing `.vmr.lock` versions (with confirmation), inject env, and
    /// enter a session shell.
    #[command(name = "cdhook", alias = "cd-hook", alias = "CdHook")]
    CdHook,
    /// Toggle whether nested sessions are allowed.
    #[command(name = "nested", visible_alias = "ns")]
    NestedSessions,
    /// Query the session mode (`VM_DISABLE`).
    #[command(name = "session", visible_alias = "se")]
    IsSessionMode,
    /// List available plugins and the SDKs they support (plain-text table).
    Plugins {
        #[arg(
            short = 'u',
            long = "update-all",
            help = "refresh all plugins from the upstream repo first"
        )]
        update_all: bool,
    },
    /// List available versions (`-p` pixi, `-n` ignore cache).
    #[command(visible_alias = "s")]
    Search {
        #[arg(help = "SDK name")]
        sdk: Option<String>,
        #[arg(
            short = 'p',
            long,
            alias = "conda",
            help = "query the pixi conda-forge backend"
        )]
        pixi: bool,
        #[arg(short = 'n', long = "no-cache", help = "ignore cached version tables")]
        no_cache: bool,
    },
    /// List installed versions; `-c` only shows the version in use.
    #[command(visible_alias = "l")]
    Local {
        #[arg(help = "SDK name (omit for every installed SDK)")]
        sdk: Option<String>,
        #[arg(short = 'c', long = "current", help = "only show the version in use")]
        current: bool,
    },
    /// Install and switch a version (`sdk@version`), or every entry in a freeze file.
    #[command(visible_aliases = ["u", "h"])]
    Use {
        #[arg(help = "sdk@version")]
        spec: Option<String>,
        #[arg(
            short = 'f',
            long = "file",
            value_name = "FILE",
            conflicts_with_all = ["spec", "session", "to_lock", "pixi"],
            help = "install every `sdk@version` listed in FILE"
        )]
        file: Option<String>,
        #[arg(short = 's', long = "session", help = "session-only install")]
        session: bool,
        #[arg(
            short = 'l',
            long = "lock",
            help = "write `.vmr.lock` and enter a session"
        )]
        to_lock: bool,
        #[arg(
            short = 'p',
            long,
            alias = "conda",
            help = "install through the pixi backend"
        )]
        pixi: bool,
    },
    /// Write installed SDK versions to `vmr_<YYYYMMDD>.txt` (or FILE).
    #[command(visible_alias = "fz")]
    Freeze {
        #[arg(help = "output file (default: vmr_<YYYYMMDD>.txt)")]
        file: Option<String>,
    },
    /// Diagnose the vmr installation, shell integration, environment and PATH.
    Doctor {
        #[arg(long, help = "machine-readable JSON output")]
        json: bool,
        #[arg(long, help = "only show warnings and errors")]
        quiet: bool,
        #[arg(long, help = "exit 1 when warnings are found")]
        strict: bool,
        #[arg(long = "no-emoji", help = "plain-text status labels")]
        no_emoji: bool,
        #[arg(long, help = "shell to check (bash, zsh, fish or powershell)")]
        shell: Option<String>,
    },
    /// Run the Model Context Protocol server on stdio.
    Mcp,
    /// Uninstall `sdk@version` or `sdk@all`.
    #[command(visible_aliases = ["uni", "r"])]
    Uninstall { spec: Option<String> },
}

fn version_str() -> &'static str {
    option_env!("VMR_VERSION").unwrap_or(concat!(env!("CARGO_PKG_VERSION"), "(dev)"))
}

fn err_exit(e: String) -> ! {
    eprintln!("error: {e}");
    std::process::exit(1);
}

fn help() {
    let mut cmd = Cli::command();
    let _ = cmd.print_help();
    println!();
}

fn main() {
    // Windows shim dispatch: `vmr.exe` is hardlinked into `~/.vmr/bin` under each SDK
    // command name, so the invoked name selects the mapped executable (no clap parsing).
    #[cfg(windows)]
    if let Some(code) = vmr_env::shims::dispatch() {
        std::process::exit(code);
    }

    // Rust ignores SIGPIPE by default, turning `vmr search ... | head` into a panic. Restore
    // the conventional Unix behavior so a closed pipe terminates the process quietly.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    // Parse before the conf → env write-back: `vmr doctor` needs the raw shell environment,
    // and `VMRConf::new` overwrites it with non-empty conf.toml values.
    let cli = Cli::parse();
    doctor::capture_raw_env();
    // `doctor` must not create `~/.vmr` or rewrite the process environment: read conf.toml
    // without the write-back that `VMRConf::new` performs for every mutating command.
    let conf = if matches!(&cli.cmd, Some(Cmd::Doctor { .. })) {
        let path = paths::conf_file_path().unwrap_or_else(paths::conf_file_default_path);
        vmr_core::conf::read_conf_file(&path).unwrap_or_default()
    } else {
        vmr_core::conf::VMRConf::new()
    };
    let Some(cmd) = cli.cmd else {
        help();
        return;
    };
    if let Err(e) = run(cmd, conf) {
        err_exit(e);
    }
}

fn run(cmd: Cmd, conf: vmr_core::conf::VMRConf) -> Result<(), String> {
    match cmd {
        Cmd::Version => {
            println!("{}", version_str());
            Ok(())
        }
        Cmd::Init { shell } => {
            let kind = match shell.as_deref() {
                Some(s) => vmr_shell::Kind::parse(s).ok_or_else(|| {
                    format!("unsupported shell: {s} (expected bash, zsh, fish or powershell)")
                })?,
                None => vmr_shell::detect_kind(),
            };
            print!("{}", vmr_shell::init_script(kind));
            print!("{}", completion_script(kind));
            Ok(())
        }
        Cmd::Template { kind, dir } => cmd_template(kind.as_deref(), dir.as_deref()),
        Cmd::NestedSessions => {
            let mut c = vmr_core::conf::VMRConf::new();
            let v = c.toggle_allow_nested_sessions();
            println!("allow nested sessions: {v}");
            Ok(())
        }
        Cmd::IsSessionMode => {
            let v = std::env::var(envs::VM_DISABLE).unwrap_or_default();
            if v.is_empty() {
                println!("Current shell is in global mode.");
            } else {
                println!("Current shell is in session mode.");
                println!(
                    "Nested sessions are not recommended. Use 'exit' to leave the session; use 'svmr' after a global install."
                );
            }
            Ok(())
        }
        Cmd::Plugins { update_all } => list_plugins(update_all),
        Cmd::Search {
            sdk,
            pixi,
            no_cache,
        } => match sdk {
            Some(sdk) => search(&sdk, pixi, no_cache),
            None => {
                help();
                Ok(())
            }
        },
        Cmd::Local { sdk, current } => match (sdk, current) {
            (Some(sdk), true) => local_current(&sdk),
            (Some(sdk), false) => local(&sdk),
            (None, true) => {
                local_current_all();
                Ok(())
            }
            (None, false) => {
                local_all();
                Ok(())
            }
        },
        Cmd::CdHook => cmd_cd_hook(),
        Cmd::Use {
            spec,
            file,
            session,
            to_lock,
            pixi,
        } => match file {
            Some(file) => cmd_use_file(&file),
            None => cmd_use(spec.as_deref(), session, to_lock, pixi),
        },
        Cmd::Freeze { file } => cmd_freeze(file.as_deref()),
        Cmd::Doctor {
            json,
            quiet,
            strict,
            no_emoji,
            shell,
        } => doctor::run(
            doctor::Options {
                json,
                quiet,
                strict,
                no_emoji,
                shell,
            },
            conf,
            version_str(),
        ),
        Cmd::Mcp => mcp::serve(),
        Cmd::Uninstall { spec } => cmd_uninstall(spec.as_deref()),
    }
}

pub(crate) fn find_plugin_for(sdk: &str) -> Option<Plugin> {
    let mut plugins = Plugins::new();
    plugins
        .get_by_plugin_name(sdk)
        .or_else(|| plugins.get_by_sdk_name(sdk))
}

pub(crate) struct PluginRow {
    pub plugin_name: String,
    pub plugin_version: String,
    pub sdk_name: String,
    pub homepage: String,
}

/// Plugin metadata used by `vmr plugins` and the MCP `vmr_list_plugins` tool.
pub(crate) fn plugin_rows(update_all: bool) -> Result<Vec<PluginRow>, String> {
    if update_all {
        vmr_lua::plugins_update::update_plugins()?;
    }
    let mut all = Plugins::new().load_all();
    if all.is_empty() {
        return Err(
            "no plugins found; check your network/proxy and run `vmr plugins -u`".to_string(),
        );
    }
    all.sort_by(|a, b| a.plugin_name.cmp(&b.plugin_name));
    Ok(all
        .into_iter()
        .map(|p| PluginRow {
            plugin_name: p.plugin_name,
            plugin_version: p.plugin_version,
            sdk_name: p.sdk_name,
            homepage: p.homepage,
        })
        .collect())
}

fn list_plugins(update_all: bool) -> Result<(), String> {
    let rows: Vec<Vec<String>> = plugin_rows(update_all)?
        .into_iter()
        .map(|p| vec![p.plugin_name, p.plugin_version, p.sdk_name, p.homepage])
        .collect();
    if update_all {
        println!("plugins updated.");
    }
    table::print_table(&["PLUGIN", "VERSION", "SDK", "HOMEPAGE"], &rows);
    Ok(())
}

pub(crate) struct VersionRow {
    pub version: String,
    pub installer: String,
    pub os: String,
    pub arch: String,
}

/// Available versions for an SDK; shared by `vmr search` and MCP.
pub(crate) fn search_versions(
    sdk: &str,
    pixi: bool,
    no_cache: bool,
) -> Result<Vec<VersionRow>, String> {
    if pixi {
        let versions = vmr_pixi::query_versions_cached(sdk, !no_cache)?;
        let (os, arch) = (
            vmr_pixi::platform::os_name(),
            vmr_pixi::platform::arch_name(),
        );
        return Ok(versions
            .into_iter()
            .map(|v| VersionRow {
                version: v,
                installer: "pixi".to_string(),
                os: os.clone(),
                arch: arch.clone(),
            })
            .collect());
    }

    let mut plugin = find_plugin_for(sdk).ok_or_else(|| format!("no plugin found for: {sdk}"))?;
    let versions = plugin.get_sdk_versions_cached(!no_cache)?;
    let mut names: Vec<String> = versions.keys().cloned().collect();
    vmr_utils::semver::sort_versions(&mut names);
    Ok(names
        .into_iter()
        .filter_map(|v| {
            versions.get(&v).map(|item| VersionRow {
                version: v,
                installer: match item.installer.as_str() {
                    "" => installer_kind::UNARCHIVER.to_string(),
                    // The package is installed through the pixi/rattler backend now.
                    installer_kind::CONDA | installer_kind::CONDA_FORGE => "pixi".to_string(),
                    other => other.to_string(),
                },
                os: item.os.clone(),
                arch: item.arch.clone(),
            })
        })
        .collect())
}

fn search(sdk: &str, pixi: bool, no_cache: bool) -> Result<(), String> {
    let rows = search_versions(sdk, pixi, no_cache)?;
    if rows.is_empty() {
        if pixi {
            println!("no versions found for {sdk} in the pixi channel.");
        } else {
            println!("no versions found for {sdk} on the current platform.");
        }
        return Ok(());
    }
    let table_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|r| vec![r.version, r.installer, r.os, r.arch])
        .collect();
    table::print_table(&["VERSION", "INSTALLER", "OS", "ARCH"], &table_rows);
    Ok(())
}

/// Installed versions of one SDK.
pub(crate) struct SdkVersions {
    /// SDK name as used in the version directory.
    pub sdk: String,
    /// Plugin name when a plugin owns the SDK, otherwise the SDK name.
    pub label: String,
    pub versions: Vec<String>,
    pub current: Option<String>,
}

/// Installed versions of one SDK: plugin metadata when available, else the disk layout.
///
/// An invalid name yields an empty result instead of a disk access: callers include `.vmr.lock`
/// entries and MCP arguments, and `sdk_version_dir` would otherwise create a directory outside
/// the versions root.
pub(crate) fn sdk_versions(sdk: &str) -> SdkVersions {
    if !is_valid_name(sdk) {
        return SdkVersions {
            sdk: sdk.to_string(),
            label: sdk.to_string(),
            versions: Vec::new(),
            current: None,
        };
    }
    if let Some(plugin) = find_plugin_for(sdk) {
        let info = finder::find_all(&plugin.sdk_name, &plugin.plugin_name);
        return SdkVersions {
            sdk: plugin.sdk_name,
            label: plugin.plugin_name,
            versions: info.installed,
            current: info.current,
        };
    }
    let root = common::sdk_version_dir(sdk);
    SdkVersions {
        sdk: sdk.to_string(),
        label: sdk.to_string(),
        versions: installed_version_names(&root, sdk),
        current: current_with_plugin(&root, sdk, sdk),
    }
}

fn local(sdk: &str) -> Result<(), String> {
    check_name("sdk name", sdk)?;
    let info = sdk_versions(sdk);
    if info.versions.is_empty() {
        let root = common::sdk_version_dir(&info.sdk);
        let empty = std::fs::read_dir(&root)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false);
        if empty {
            let _ = std::fs::remove_dir_all(&root);
        }
        println!("(no versions installed for {})", info.label);
        return Ok(());
    }
    let rows: Vec<Vec<String>> = info
        .versions
        .iter()
        .map(|v| {
            let mark = if info.current.as_deref() == Some(v.as_str()) {
                "<current>"
            } else {
                ""
            };
            vec![v.clone(), mark.to_string()]
        })
        .collect();
    table::print_table(&["VERSION", "STATUS"], &rows);
    Ok(())
}

/// `local -c <sdk>`: only the version in use, printed alone (script-friendly); prints
/// nothing when the SDK is not installed or has no current version.
fn local_current(sdk: &str) -> Result<(), String> {
    check_name("sdk name", sdk)?;
    if let Some(cur) = sdk_versions(sdk).current {
        println!("{cur}");
    }
    Ok(())
}

/// Version root directories (`<sdk>_versions`) that contain at least one installed version.
///
/// Read-only commands such as `local` may create an empty version root as a side effect;
/// those are removed here so they do not show up as "installed SDKs".
fn version_roots() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(paths::versions_dir()) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(sdk) = name.strip_suffix(common::VERSION_DIR_SUFFIX) else {
                continue;
            };
            if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let has_versions = std::fs::read_dir(e.path())
                .map(|it| {
                    it.flatten()
                        .any(|x| x.file_type().map(|t| t.is_dir()).unwrap_or(false))
                })
                .unwrap_or(false);
            if has_versions {
                out.push((sdk.to_string(), e.path()));
            } else {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Installed versions for every SDK that has at least one version on disk.
pub(crate) fn installed_versions_all() -> Vec<SdkVersions> {
    let plugins = Plugins::new().load_all();
    version_roots()
        .into_iter()
        .map(|(sdk, root)| {
            if let Some(p) = plugins.iter().find(|p| p.sdk_name == sdk) {
                let info = finder::find_all(&p.sdk_name, &p.plugin_name);
                SdkVersions {
                    sdk: p.sdk_name.clone(),
                    label: p.plugin_name.clone(),
                    versions: info.installed,
                    current: info.current,
                }
            } else {
                SdkVersions {
                    sdk: sdk.clone(),
                    label: sdk.clone(),
                    versions: installed_version_names(&root, &sdk),
                    current: current_with_plugin(&root, &sdk, &sdk),
                }
            }
        })
        .collect()
}

/// `local -c` without an SDK: every installed SDK with the version in use; prints nothing
/// when no SDK is installed.
fn local_current_all() {
    let mut rows: Vec<Vec<String>> = installed_versions_all()
        .into_iter()
        .map(|e| vec![e.sdk, e.current.unwrap_or_default()])
        .collect();
    if rows.is_empty() {
        return;
    }
    rows.sort();
    table::print_table(&["SDK", "CURRENT"], &rows);
}

/// `local` without an SDK argument: per-SDK version blocks for every installed SDK.
fn local_all() {
    let entries = installed_versions_all();
    if entries.is_empty() {
        println!("(no SDKs installed)");
        return;
    }
    for entry in entries {
        println!("[{}]", entry.sdk);
        if entry.versions.is_empty() {
            println!("  (none)");
            continue;
        }
        for version in entry.versions {
            let mark = if entry.current.as_deref() == Some(version.as_str()) {
                " <current>"
            } else {
                ""
            };
            println!("  {version}{mark}");
        }
    }
}

/// `vmr cdhook`: install `.vmr.lock` versions that are missing, then inject env from the lock
/// and, when not already in a session, enter a child shell (mirrors Go `HookForCdCommand` +
/// `RunTerminal`). Called by the shell cd hook.
fn cmd_cd_hook() -> Result<(), String> {
    // Resolve the lock once: the shell hook caches the project directory written here and
    // stays silent until the shell leaves it, so this search runs only on a project switch.
    let locker = locker::VersionLocker::load_from(None);
    locker.write_cd_cache();
    // The outer hook follows this file after every `vmr cdhook`; drop anything left by a
    // session whose parent never consumed it (killed shell) or by an earlier interrupted run.
    let _ = std::fs::remove_file(paths::session_exit_file());
    install_missing_locked_versions(&locker)?;
    let action = ins::hook_for_cd_command(&locker)?;
    if let ins::Action::RunSession = action {
        run_lock_session(locker.lock_dir());
    }
    Ok(())
}

/// Enters the `.vmr.lock` child shell, recording the project root so the session hook can
/// leave the shell when `cd` goes outside it.
fn run_lock_session(project: Option<&Path>) -> ! {
    match project {
        Some(dir) => unsafe { std::env::set_var(envs::SESSION_PROJECT, dir) },
        None => unsafe { std::env::remove_var(envs::SESSION_PROJECT) },
    }
    // The shell that installs the session hook exports this marker; start every hook session
    // without it so inherited subshells do not suppress it.
    unsafe { std::env::remove_var(envs::SESSION_ACTIVE) };
    std::process::exit(vmr_pty::run_terminal())
}

/// Offers to install versions pinned by `.vmr.lock` that are not on disk.
///
/// Declining, or a non-interactive stdin, skips the install; the hook then injects only the
/// versions that are installed. Confirmed installs stay inactive (no symlink/global env);
/// the hook exposes them to the session through temporary env entries.
fn install_missing_locked_versions(locker: &locker::VersionLocker) -> Result<(), String> {
    let missing = ins::missing_locked_versions(locker);
    if missing.is_empty() {
        return Ok(());
    }
    if !confirm_install_locked(&missing) {
        eprintln!("skipped installing locked version(s) from .vmr.lock.");
        return Ok(());
    }
    let mut failed = Vec::new();
    for (sdk, version) in &missing {
        let spec = format!("{sdk}@{version}");
        // A lock entry without a Lua plugin is a pixi/conda install, matching the layout
        // fallback in `hook_for_cd_command`.
        let pixi = find_plugin_for(sdk).is_none();
        let result =
            build_install_request(&spec, true, false, pixi).and_then(|req| ins::install_only(&req));
        match result {
            Ok(()) => println!("{sdk} {version} installed."),
            Err(e) => {
                eprintln!("failed to install {spec}: {e}");
                failed.push(spec);
            }
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} of {} locked version(s) failed: {}",
            failed.len(),
            missing.len(),
            failed.join(", ")
        ))
    }
}

/// Shows the missing lock entries and asks whether to install them. Anything but `y`/`yes`
/// (including EOF) declines. A non-terminal stdin is never prompted.
fn confirm_install_locked(missing: &[(String, String)]) -> bool {
    use std::io::{IsTerminal, Write};

    eprintln!(".vmr.lock pins version(s) that are not installed:");
    for (sdk, version) in missing {
        eprintln!("  {sdk} {version}");
    }
    if !std::io::stdin().is_terminal() {
        eprintln!("stdin is not a terminal; not installing them.");
        return false;
    }

    eprint!("Install now? [y/N] ");
    let _ = std::io::stderr().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or(0) > 0 && is_yes(&input)
}

/// Accepts `y`/`yes` in any case, ignoring surrounding whitespace.
fn is_yes(input: &str) -> bool {
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Outcome of a global install/switch, shared by the CLI and MCP.
pub(crate) struct InstallSummary {
    pub sdk_name: String,
    pub plugin_name: String,
    pub version: String,
    pub path: PathBuf,
    pub pixi: bool,
    pub already_installed: bool,
}

fn dir_has_entries(path: &Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

/// Resolves `sdk@version` into an install request (shared by CLI and MCP).
fn build_install_request(
    spec: &str,
    session: bool,
    to_lock: bool,
    pixi: bool,
) -> Result<ins::InstallRequest, String> {
    let (sdk, version) = match spec.split_once('@') {
        Some((s, v)) if !s.is_empty() && !v.is_empty() => (s.to_string(), v.to_string()),
        _ => {
            return Err("usage: vmr use <sdk>@<version>".to_string());
        }
    };
    // sdk/version/plugin names become path components under ~/.vmr/versions.
    check_name("sdk name", &sdk)?;
    check_name("version", &version)?;

    let mut plugin = find_plugin_for(&sdk);
    if plugin.is_none() && !pixi {
        return Err(format!("no plugin found for: {sdk}"));
    }

    let version = if version == "latest" {
        plugin
            .as_mut()
            .ok_or_else(|| format!("no plugin found for: {sdk}"))?
            .sorted_versions_result()?
            .into_iter()
            .next()
            .ok_or_else(|| format!("no versions found for {sdk}"))?
    } else {
        version
    };
    // `latest` resolves against remote version metadata, so this is not covered above.
    check_name("version", &version)?;

    let item = if pixi {
        Item {
            os: vmr_pixi::platform::os_name(),
            arch: vmr_pixi::platform::arch_name(),
            installer: installer_kind::CONDA.to_string(),
            ..Default::default()
        }
    } else {
        plugin
            .as_mut()
            .expect("checked above")
            .get_version_result(&version)?
            .ok_or_else(|| format!("version not found: {sdk}@{version}"))?
    };

    // A pixi/conda install always uses the conda prefix layout. Do not reuse the plugin's
    // `InstallerConfig`: it describes the upstream archive (e.g. Go's `GOROOT` layout), not
    // the conda prefix, even when a plugin with the same SDK name exists.
    let ic = if pixi {
        pixi_env::installer_config()
    } else {
        match plugin.as_mut() {
            Some(p) => p.get_installer_config()?,
            None => vmr_lua::types::InstallerConfig::default(),
        }
    };
    let (sdk_name, plugin_name) = match &plugin {
        Some(p) => (p.sdk_name.clone(), p.plugin_name.clone()),
        None => (sdk.clone(), sdk.clone()),
    };
    Ok(ins::InstallRequest {
        sdk_name,
        plugin_name,
        version_name: version,
        version: item,
        ic,
        mode: if session {
            ins::InvokeMode::Sessionly
        } else if to_lock {
            ins::InvokeMode::ToLock
        } else {
            ins::InvokeMode::Globally
        },
    })
}

fn summarize(req: &ins::InstallRequest, pixi: bool, already_installed: bool) -> InstallSummary {
    InstallSummary {
        sdk_name: req.sdk_name.clone(),
        plugin_name: req.plugin_name.clone(),
        version: req.version_name.clone(),
        path: req.symbol_path(),
        pixi,
        already_installed,
    }
}

/// Installs and globally activates `sdk@version` without printing (used by MCP).
pub(crate) fn install_spec(spec: &str, pixi: bool) -> Result<InstallSummary, String> {
    let req = build_install_request(spec, false, false, pixi)?;
    let already_installed = dir_has_entries(&req.install_dir());
    match ins::install(&req)? {
        ins::Action::Done => Ok(summarize(&req, pixi, already_installed)),
        ins::Action::RunSession => {
            Err("session installs are not supported in this mode".to_string())
        }
    }
}

fn cmd_use(spec: Option<&str>, session: bool, to_lock: bool, pixi: bool) -> Result<(), String> {
    let Some(spec) = spec else {
        help();
        return Ok(());
    };
    let req = build_install_request(spec, session, to_lock, pixi)?;
    let summary = summarize(&req, pixi, dir_has_entries(&req.install_dir()));
    match ins::install(&req)? {
        ins::Action::Done => {
            println!(
                "{} {} is now active ({})",
                summary.plugin_name,
                summary.version,
                summary.path.display()
            );
            Ok(())
        }
        ins::Action::RunSession => {
            // `use -s`/`use -l` are not cd-hook project sessions; never inherit markers.
            unsafe {
                std::env::remove_var(envs::SESSION_PROJECT);
                std::env::remove_var(envs::SESSION_ACTIVE);
            }
            std::process::exit(vmr_pty::run_terminal())
        }
    }
}

/// One `sdk@version` line from a freeze file; `pixi` selects the pixi backend on replay.
struct FreezeEntry {
    spec: String,
    pixi: bool,
}

/// Parses a freeze file: blank lines and `#` comments are ignored; each entry is
/// `sdk@version` with an optional trailing `pixi` backend token.
fn parse_freeze_file(data: &str) -> Result<Vec<FreezeEntry>, String> {
    let mut out = Vec::new();
    for (idx, raw) in data.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let spec = parts.next().expect("non-empty line");
        let (sdk, version) = spec
            .split_once('@')
            .ok_or_else(|| format!("line {}: expected `sdk@version`, got `{spec}`", idx + 1))?;
        if sdk.is_empty() || version.is_empty() {
            return Err(format!(
                "line {}: expected `sdk@version`, got `{spec}`",
                idx + 1
            ));
        }
        check_name("sdk name", sdk).map_err(|e| format!("line {}: {e}", idx + 1))?;
        check_name("version", version).map_err(|e| format!("line {}: {e}", idx + 1))?;
        let pixi = match parts.next() {
            None => false,
            Some("pixi") => true,
            Some(other) => {
                return Err(format!(
                    "line {}: unknown backend `{other}` (expected `pixi`)",
                    idx + 1
                ));
            }
        };
        if parts.next().is_some() {
            return Err(format!("line {}: unexpected extra fields", idx + 1));
        }
        out.push(FreezeEntry {
            spec: spec.to_string(),
            pixi,
        });
    }
    if out.is_empty() {
        return Err("no entries found in freeze file".to_string());
    }
    Ok(out)
}

/// `vmr use -f FILE`: installs every entry, continuing past failures.
fn cmd_use_file(file: &str) -> Result<(), String> {
    let data = std::fs::read_to_string(file).map_err(|e| format!("read {file} failed: {e}"))?;
    let entries = parse_freeze_file(&data)?;
    println!("installing {} SDK version(s) from {file}", entries.len());
    let mut failed = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, entries.len(), entry.spec);
        if let Err(e) = cmd_use(Some(&entry.spec), false, false, entry.pixi) {
            eprintln!("failed: {}: {e}", entry.spec);
            failed.push(entry.spec.clone());
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} of {} entr{} failed: {}",
            failed.len(),
            entries.len(),
            if failed.len() == 1 { "y" } else { "ies" },
            failed.join(", ")
        ))
    }
}

/// Installed version directory names for an SDK, prefix stripped and semver-sorted.
fn installed_version_names(root: &Path, plugin_name: &str) -> Vec<String> {
    let prefix = format!("{plugin_name}-");
    let mut versions: Vec<String> = std::fs::read_dir(root)
        .map(|it| {
            it.flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    name.strip_prefix(&prefix).map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default();
    vmr_utils::semver::sort_versions(&mut versions);
    versions
}

/// Current version of an SDK, using the plugin name for the directory prefix.
fn current_with_plugin(root: &Path, sdk: &str, plugin_name: &str) -> Option<String> {
    let target = std::fs::read_link(root.join(sdk)).ok()?;
    let name = target.file_name()?.to_string_lossy().into_owned();
    name.strip_prefix(&format!("{plugin_name}-"))
        .map(str::to_string)
}

/// Entries for `vmr freeze`, grouped by SDK; the current version is moved last so a
/// sequential replay leaves it active. SDKs without a plugin are marked `pixi`.
fn collect_freeze_entries() -> Vec<FreezeEntry> {
    let mut out = Vec::new();
    let plugins = Plugins::new().load_all();
    for (sdk, root) in version_roots() {
        let plugin_name = plugins
            .iter()
            .find(|p| p.sdk_name == sdk)
            .map(|p| p.plugin_name.clone());
        let pixi = plugin_name.is_none();
        let plugin_name = plugin_name.unwrap_or_else(|| sdk.clone());
        let mut versions = installed_version_names(&root, &plugin_name);
        if versions.is_empty() {
            continue;
        }
        if let Some(current) = current_with_plugin(&root, &sdk, &plugin_name) {
            if let Some(pos) = versions.iter().position(|v| v == &current) {
                versions.remove(pos);
                versions.push(current);
            }
        }
        for version in versions {
            out.push(FreezeEntry {
                spec: format!("{sdk}@{version}"),
                pixi,
            });
        }
    }
    out
}

/// Writes the freeze file; returns its path and entry count (printing is the caller's job).
pub(crate) fn freeze_to(file: Option<&str>) -> Result<(PathBuf, usize), String> {
    let now = chrono::Local::now();
    let path = match file.map(str::trim).filter(|f| !f.is_empty()) {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from(format!("vmr_{}.txt", now.format("%Y%m%d"))),
    };
    let entries = collect_freeze_entries();
    if entries.is_empty() {
        return Err("no SDK versions installed; nothing to freeze".to_string());
    }
    let mut text = format!(
        "# vmr freeze {}\n# replay with: vmr use -f {}\n# the last entry of each SDK becomes the active version\n",
        now.format("%Y-%m-%d"),
        path.display()
    );
    for entry in &entries {
        text.push_str(&entry.spec);
        if entry.pixi {
            text.push_str(" pixi");
        }
        text.push('\n');
    }
    std::fs::write(&path, text).map_err(|e| format!("write {} failed: {e}", path.display()))?;
    Ok((path, entries.len()))
}

fn cmd_freeze(file: Option<&str>) -> Result<(), String> {
    let (path, count) = freeze_to(file)?;
    println!("froze {count} SDK version(s) to {}", path.display());
    Ok(())
}

/// Uninstalls `sdk@version` / `sdk@all`; returns the status message (printing is the
/// caller's job).
pub(crate) fn uninstall_spec(spec: &str) -> Result<String, String> {
    let (sdk, version) = match spec.split_once('@') {
        Some((s, v)) if !s.is_empty() && !v.is_empty() => (s.to_string(), v.to_string()),
        _ => {
            return Err("usage: vmr uninstall <sdk>@<version>|all".to_string());
        }
    };
    // `all` schedules a `remove_dir_all` on the version root, so the name decides what is deleted.
    check_name("sdk name", &sdk)?;
    check_name("version", &version)?;

    let mut plugin = find_plugin_for(&sdk);

    if version == "all" {
        let (sdk_name, plugin_name) = match &plugin {
            Some(p) => (p.sdk_name.clone(), p.plugin_name.clone()),
            None => (sdk.clone(), sdk.clone()),
        };
        ins::uninstall_all(&sdk_name, &plugin_name);
        return Ok(format!("all versions of {plugin_name} uninstalled."));
    }

    // A conda-only install has no Lua plugin/version metadata; still remove its directory.
    let item = match plugin.as_mut() {
        Some(p) => p.get_version_result(&version)?.unwrap_or_default(),
        None => Item::default(),
    };
    let (sdk_name, plugin_name) = match &plugin {
        Some(p) => (p.sdk_name.clone(), p.plugin_name.clone()),
        None => (sdk.clone(), sdk.clone()),
    };
    // Unset the entries that `vmr use` actually wrote. A pixi marker means the conda layout,
    // even when a plugin with the same SDK name exists (`vmr use -p go@...`); a no-plugin
    // install has no archive layout to fall back to, so also use the pixi layout.
    let install = common::install_dir(&sdk_name, &plugin_name, &version);
    let ic = if pixi_env::is_pixi_prefix(&install) || plugin.is_none() {
        pixi_env::installer_config()
    } else {
        plugin
            .as_mut()
            .expect("plugin checked above")
            .get_installer_config()?
    };
    let req = ins::InstallRequest {
        sdk_name,
        plugin_name: plugin_name.clone(),
        version_name: version.clone(),
        version: item,
        ic,
        mode: ins::InvokeMode::Globally,
    };
    ins::uninstall(&req)?;
    Ok(format!("{plugin_name} {version} uninstalled."))
}

fn cmd_uninstall(spec: Option<&str>) -> Result<(), String> {
    let Some(spec) = spec else {
        help();
        return Ok(());
    };
    println!("{}", uninstall_spec(spec)?);
    Ok(())
}

fn cmd_template(kind: Option<&str>, dir: Option<&str>) -> Result<(), String> {
    let Some(kind) = kind else {
        return Err("usage: vmr template <github|html|json|pixi> [--dir <dir>]".to_string());
    };
    let (name, body) = match kind.to_ascii_lowercase().as_str() {
        "github" | "gh" => ("github", include_str!("../templates/github.lua")),
        "html" => ("html", include_str!("../templates/html.lua")),
        "json" => ("json", include_str!("../templates/json.lua")),
        "pixi" | "conda" => ("pixi", include_str!("../templates/pixi.lua")),
        other => {
            return Err(format!(
                "unknown template: {other} (expected github, html, json or pixi)"
            ));
        }
    };
    match dir {
        Some(dir) => {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let path = dir.join(format!("{name}.lua"));
            std::fs::write(&path, body).map_err(|e| e.to_string())?;
            println!("wrote {}", path.display());
        }
        None => print!("{body}"),
    }
    Ok(())
}

/// Completion script appended to `vmr init` output (clap-generated).
///
/// zsh `compdef` is deferred: rc files may evaluate `vmr init` before `compinit` (or never
/// run `compinit` at all), so registration happens on the first prompt instead of silently
/// being skipped. If no completion system is loaded by then, `compinit` is initialized.
/// clap_complete emits `using namespace ...` at the top of the PowerShell script. `vmr init`
/// output is evaluated with `Invoke-Expression` after other profile statements, where a
/// `using` statement is a parse error. Remove those directives and fully qualify the four
/// referenced types so the completion script can be embedded in the profile output.
fn strip_powershell_usings(script: String) -> String {
    script
        .lines()
        .filter(|line| !line.trim_start().starts_with("using namespace "))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "[StringConstantExpressionAst]",
            "[System.Management.Automation.Language.StringConstantExpressionAst]",
        )
        .replace(
            "[StringConstantType]",
            "[System.Management.Automation.Language.StringConstantType]",
        )
        .replace(
            "[CompletionResultType]",
            "[System.Management.Automation.CompletionResultType]",
        )
        .replace(
            "[CompletionResult]",
            "[System.Management.Automation.CompletionResult]",
        )
}

fn completion_script(kind: vmr_shell::Kind) -> String {
    use clap_complete::Shell;
    const ZSH_TAIL: &str = vmr_shell::ZSH_COMPLETION_TAIL;
    let shell = match kind {
        vmr_shell::Kind::Bash => Shell::Bash,
        vmr_shell::Kind::Zsh => Shell::Zsh,
        vmr_shell::Kind::Fish => Shell::Fish,
        vmr_shell::Kind::PowerShell => Shell::PowerShell,
    };
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, name, &mut buf);
    let script = String::from_utf8_lossy(&buf).into_owned();
    if matches!(kind, vmr_shell::Kind::Zsh) {
        script.replace(
            "if [ \"$funcstack[1]\" = \"_vmr\" ]; then\n    _vmr \"$@\"\nelse\n    compdef _vmr vmr\nfi",
            ZSH_TAIL,
        )
    } else if matches!(kind, vmr_shell::Kind::PowerShell) {
        strip_powershell_usings(script)
    } else {
        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_yes_confirmations() {
        assert!(is_yes("y\n"));
        assert!(is_yes("YES"));
        assert!(is_yes("  Yes \r\n"));
        assert!(!is_yes(""));
        assert!(!is_yes("\n"));
        assert!(!is_yes("n\n"));
        assert!(!is_yes("no\n"));
        assert!(!is_yes("yeah\n"));
    }

    #[test]
    fn parses_freeze_file_entries_and_comments() {
        let data = "# vmr freeze 2026-11-04\r\n\r\ngo@1.22.0\r\nlua@5.4.6 pixi\n# tail\n";
        let entries = parse_freeze_file(data).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].spec, "go@1.22.0");
        assert!(!entries[0].pixi);
        assert_eq!(entries[1].spec, "lua@5.4.6");
        assert!(entries[1].pixi);
    }

    #[test]
    fn rejects_malformed_freeze_files() {
        assert!(parse_freeze_file("go\n").is_err());
        assert!(parse_freeze_file("go@\n").is_err());
        assert!(parse_freeze_file("@1.0\n").is_err());
        assert!(parse_freeze_file("go@1.0 conda\n").is_err());
        assert!(parse_freeze_file("go@1.0 pixi extra\n").is_err());
        assert!(parse_freeze_file("\n# only comments\n").is_err());
    }

    #[test]
    fn rejects_names_that_are_not_path_components() {
        // Freeze files ship inside repositories, so an entry decides which path gets written or
        // removed; `..` and separators must not reach `sdk_version_dir`/`install_dir`.
        assert!(parse_freeze_file("../../projects/app@1\n").is_err());
        assert!(parse_freeze_file("go@1/../../x\n").is_err());
        assert!(parse_freeze_file("go@1 pixi\n").is_ok());
    }

    #[test]
    fn zsh_completion_defers_compdef_registration() {
        let script = completion_script(vmr_shell::Kind::Zsh);
        assert!(script.contains("add-zsh-hook precmd _vmr_register_completion"));
        assert!(script.contains("autoload -Uz compinit && compinit -i"));
        assert!(!script.contains("else\n    compdef _vmr vmr\nfi"));
    }

    #[test]
    fn bash_completion_registers_complete() {
        let script = completion_script(vmr_shell::Kind::Bash);
        assert!(script.contains("complete -F _vmr"));
    }

    #[test]
    fn powershell_completion_has_no_using_directives() {
        let script = completion_script(vmr_shell::Kind::PowerShell);
        assert!(!script.contains("using namespace"), "{script}");
        assert!(
            script.contains("[System.Management.Automation.CompletionResult]"),
            "{script}"
        );
        assert!(
            script.contains("[System.Management.Automation.Language.StringConstantExpressionAst]"),
            "{script}"
        );
        assert!(script.contains("Register-ArgumentCompleter"), "{script}");
    }
}
