//! vmr CLI（clap；命令/别名对齐 plan §3.11 与 Go cmd/vmr 表，无 TUI）。

mod table;

use std::path::Path;

use clap::{CommandFactory, Parser, Subcommand};

use vmr_core::envs;
use vmr_core::paths;
use vmr_installer::{common, finder, installer as ins};
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
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(alias = "v")]
    Version,
    /// 设置本地代理。
    #[command(alias = "sp")]
    SetProxy { uri: String },
    /// 设置反向代理。
    #[command(alias = "sr", alias = "srp")]
    SetReverseProxy { uri: String },
    /// 设置下载线程数。
    #[command(alias = "sdt", alias = "st")]
    SetDownloadThreads { num: i32 },
    /// 切换自定义镜像开关。
    #[command(alias = "tcm", alias = "tm")]
    ToggleCustomedMirrors,
    /// 允许嵌套会话开关。
    #[command(alias = "ns")]
    NestedSessions,
    /// 查询会话模式（VM_DISABLE）。
    #[command(alias = "ism")]
    IsSessionMode,
    /// 列出可用 SDK（文本表格）。
    #[command(alias = "S")]
    Show,
    /// 查询某 SDK 的版本列表；-c 走 conda 源。
    #[command(alias = "s")]
    Search {
        sdk: String,
        #[arg(short = 'c', long)]
        conda: bool,
    },
    /// 显示某 SDK 已装版本（当前标 <current>）。
    #[command(alias = "l")]
    Local { sdk: String },
    /// 列出已装 SDK。
    #[command(alias = "in")]
    InstalledSdks,
    /// 每个已装 SDK 的版本信息。
    #[command(alias = "ii")]
    InstalledInfo,
    /// 安装并切换 SDK 版本（sdk@version）；-E 项目锁 / -s 会话 / -l 锁版本 / -c conda。
    #[command(alias = "u", alias = "h")]
    Use {
        spec: String,
        #[arg(short = 'E')]
        cd_hook: bool,
        #[arg(short = 's')]
        session: bool,
        #[arg(short = 'l')]
        to_lock: bool,
        #[arg(short = 'c')]
        conda: bool,
    },
    /// 卸载 sdk@version 或 @all。
    #[command(alias = "uni", alias = "r")]
    Uninstall { spec: String },
    /// 更新 Lua 插件。
    #[command(alias = "up")]
    UpdatePlugins,
    /// 安装 shell 补全（bash/zsh/fish/powershell）。
    #[command(alias = "ac")]
    AddCompletions { shell: String },
    /// 自安装。
    #[command(alias = "i", alias = "is")]
    InstallSelf {
        #[arg(long)]
        sdk_dir: Option<String>,
    },
    /// 自卸载（供脚本调用）。
    #[command(alias = "Uins")]
    UninstallSelf,
}

fn version_str() -> &'static str {
    option_env!("VMR_VERSION").unwrap_or("0.1.0(dev)")
}

fn err_exit(e: String) -> ! {
    eprintln!("error: {e}");
    std::process::exit(1);
}

fn main() {
    // conf → env 回写（env 为运行时权威，对齐 Go 副作用初始化）。
    let _ = vmr_core::conf::VMRConf::new();
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        err_exit(e);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.cmd {
        Cmd::Version => {
            println!("{}", version_str());
            Ok(())
        }
        Cmd::SetProxy { uri } => {
            let mut c = vmr_core::conf::VMRConf::new();
            c.set_proxy_uri(&uri);
            println!("local proxy: {uri}");
            Ok(())
        }
        Cmd::SetReverseProxy { uri } => {
            let mut c = vmr_core::conf::VMRConf::new();
            c.set_reverse_proxy(&uri);
            println!("reverse proxy: {uri}");
            Ok(())
        }
        Cmd::SetDownloadThreads { num } => {
            let mut c = vmr_core::conf::VMRConf::new();
            c.set_download_thread_num(num);
            println!("download threads: {}", num.max(1));
            Ok(())
        }
        Cmd::ToggleCustomedMirrors => {
            let mut c = vmr_core::conf::VMRConf::new();
            c.toggle_use_customed_mirrors();
            println!("use customed mirrors: {}", c.use_customed_mirrors);
            Ok(())
        }
        Cmd::NestedSessions => {
            let mut c = vmr_core::conf::VMRConf::new();
            let v = c.toggle_allow_nested_sessions();
            println!("allow nested sessions: {v}");
            Ok(())
        }
        Cmd::IsSessionMode => {
            let v = std::env::var(envs::VM_DISABLE).unwrap_or_default();
            println!("{v}");
            Ok(())
        }
        Cmd::Show => {
            show();
            Ok(())
        }
        Cmd::Search { sdk, conda } => search(&sdk, conda),
        Cmd::Local { sdk } => local(&sdk),
        Cmd::InstalledSdks => {
            installed_sdks();
            Ok(())
        }
        Cmd::InstalledInfo => {
            installed_info();
            Ok(())
        }
        Cmd::Use {
            spec,
            cd_hook,
            session,
            to_lock,
            conda,
        } => cmd_use(&spec, cd_hook, session, to_lock, conda),
        Cmd::Uninstall { spec } => cmd_uninstall(&spec),
        Cmd::UpdatePlugins => {
            vmr_lua::plugins_update::update_plugins()?;
            println!("plugins updated.");
            Ok(())
        }
        Cmd::AddCompletions { shell } => cmd_add_completions(&shell),
        Cmd::InstallSelf { sdk_dir } => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            vmr_self::install_self(&exe, sdk_dir.as_deref())?;
            println!(
                "vmr installed to {}",
                vmr_self::installed_bin_path().display()
            );
            Ok(())
        }
        Cmd::UninstallSelf => {
            vmr_self::uninstall_self()?;
            println!("vmr self removed (run: rm -rf ~/.vmr to fully clean).");
            Ok(())
        }
    }
}

fn find_plugin_for(sdk: &str) -> Option<Plugin> {
    let mut plugins = Plugins::new();
    plugins
        .get_by_plugin_name(sdk)
        .or_else(|| plugins.get_by_sdk_name(sdk))
}

fn show() {
    let mut plugins = Plugins::new();
    let mut all = plugins.load_all();
    all.sort_by(|a, b| a.plugin_name.cmp(&b.plugin_name));
    let rows: Vec<Vec<String>> = all
        .iter()
        .map(|p| {
            vec![
                p.plugin_name.clone(),
                p.plugin_version.clone(),
                p.sdk_name.clone(),
                p.homepage.clone(),
            ]
        })
        .collect();
    table::print_table(&["PLUGIN", "VERSION", "SDK", "HOMEPAGE"], &rows);
}

fn search(sdk: &str, conda: bool) -> Result<(), String> {
    if conda {
        let versions = vmr_conda::query_versions(sdk)?;
        let (os, arch) = (
            vmr_conda::platform::os_name(),
            vmr_conda::platform::arch_name(),
        );
        let rows: Vec<Vec<String>> = versions
            .iter()
            .map(|v| vec![v.clone(), "conda".into(), os.clone(), arch.clone()])
            .collect();
        table::print_table(&["VERSION", "INSTALLER", "OS", "ARCH"], &rows);
        return Ok(());
    }
    let mut plugin = find_plugin_for(sdk).ok_or_else(|| format!("no plugin found for: {sdk}"))?;
    let versions = plugin.sorted_versions();
    let rows: Vec<Vec<String>> = versions
        .iter()
        .filter_map(|v| {
            plugin.get_version(v).map(|item| {
                vec![
                    v.clone(),
                    if item.installer.is_empty() {
                        installer_kind::UNARCHIVER.into()
                    } else {
                        item.installer.clone()
                    },
                    item.os.clone(),
                    item.arch.clone(),
                    item.url.clone(),
                ]
            })
        })
        .collect();
    table::print_table(&["VERSION", "INSTALLER", "OS", "ARCH", "URL"], &rows);
    Ok(())
}

fn local(sdk: &str) -> Result<(), String> {
    let plugin = find_plugin_for(sdk).ok_or_else(|| format!("no plugin found for: {sdk}"))?;
    let plugin_name = plugin.plugin_name.clone();
    let sdk_name = plugin.sdk_name.clone();
    let info = finder::find_all(&sdk_name, &plugin_name);
    let rows: Vec<Vec<String>> = info
        .installed
        .iter()
        .map(|v| {
            let mark = if info.current.as_deref() == Some(v) {
                "<current>"
            } else {
                ""
            };
            vec![v.clone(), mark.to_string()]
        })
        .collect();
    if rows.is_empty() {
        println!("(no versions installed for {})", plugin_name);
        return Ok(());
    }
    table::print_table(&["VERSION", "STATUS"], &rows);
    Ok(())
}

fn installed_sdks() {
    let vdir = paths::versions_dir();
    let mut rows = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&vdir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(sdk) = name.strip_suffix(common::VERSION_DIR_SUFFIX) {
                let sym = vdir.join(&name).join(sdk);
                let current = std::fs::read_link(sym)
                    .ok()
                    .and_then(|t| t.file_name().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_default();
                rows.push(vec![sdk.to_string(), current]);
            }
        }
    }
    rows.sort();
    table::print_table(&["SDK", "CURRENT"], &rows);
}

fn installed_info() {
    let vdir = paths::versions_dir();
    if let Ok(entries) = std::fs::read_dir(&vdir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(sdk) = name.strip_suffix(common::VERSION_DIR_SUFFIX) else {
                continue;
            };
            let root = e.path();
            let sym = root.join(sdk);
            let current = std::fs::read_link(&sym)
                .ok()
                .and_then(|t| t.file_name().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let mut versions: Vec<String> = std::fs::read_dir(&root)
                .map(|it| {
                    it.flatten()
                        .filter(|x| x.file_type().map(|t| t.is_dir()).unwrap_or(false))
                        .map(|x| x.file_name().to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or_default();
            versions.sort();
            let mut rows = Vec::new();
            for v in versions {
                let mark = if current.ends_with(&format!("-{v}")) || v == current {
                    "<current>"
                } else {
                    ""
                };
                rows.push(vec![v, mark.to_string()]);
            }
            println!("[{}]", sdk);
            if rows.is_empty() {
                println!("  (none)");
            } else {
                for r in rows {
                    println!("  {} {}", r[0], r[1]);
                }
            }
        }
    }
}

fn parse_spec(spec: &str) -> (String, String) {
    match spec.split_once('@') {
        Some((s, v)) => (s.to_string(), v.to_string()),
        None => (spec.to_string(), String::new()),
    }
}

fn cmd_use(
    spec: &str,
    cd_hook: bool,
    session: bool,
    to_lock: bool,
    conda: bool,
) -> Result<(), String> {
    if cd_hook {
        // vmr use -E：按锁注入并进入会话（对齐 Go HookForCdCommand）。
        let action = ins::hook_for_cd_command()?;
        if let ins::Action::RunSession = action {
            std::process::exit(vmr_pty::run_terminal());
        }
        return Ok(());
    }
    let (sdk, version) = parse_spec(spec);
    if sdk.is_empty() || version.is_empty() {
        return Err("usage: vmr use <sdk>@<version>".to_string());
    }
    let mut plugin = find_plugin_for(&sdk).ok_or_else(|| format!("no plugin found for: {sdk}"))?;
    let item = if conda {
        Item {
            os: vmr_conda::platform::os_name(),
            arch: vmr_conda::platform::arch_name(),
            installer: installer_kind::CONDA.to_string(),
            ..Default::default()
        }
    } else {
        plugin
            .get_version(&version)
            .ok_or_else(|| format!("version not found: {sdk}@{version}"))?
    };
    let ic = plugin.get_installer_config()?;
    let req = ins::InstallRequest {
        sdk_name: plugin.sdk_name.clone(),
        plugin_name: plugin.plugin_name.clone(),
        version_name: version.clone(),
        version: item.clone(),
        ic,
        mode: if session {
            ins::InvokeMode::Sessionly
        } else if to_lock {
            ins::InvokeMode::ToLock
        } else {
            ins::InvokeMode::Globally
        },
        no_envs: conda, // conda 安装提示手动加 PATH（Go 行为）。
    };
    let action = ins::install(&req)?;
    match action {
        ins::Action::Done => {
            if conda {
                let sym = req.symbol_path();
                println!(
                    "conda sdk installed; add to PATH manually: {}",
                    sym.display()
                );
            } else {
                let sym = req.symbol_path();
                println!(
                    "{} {} is now active ({})",
                    plugin.plugin_name,
                    version,
                    sym.display()
                );
            }
            Ok(())
        }
        ins::Action::RunSession => std::process::exit(vmr_pty::run_terminal()),
    }
}

fn cmd_uninstall(spec: &str) -> Result<(), String> {
    let (sdk, version) = parse_spec(spec);
    if sdk.is_empty() {
        return Err("usage: vmr uninstall <sdk>@<version>|all".to_string());
    }
    let mut plugin = find_plugin_for(&sdk).ok_or_else(|| format!("no plugin found for: {sdk}"))?;
    let ic = plugin.get_installer_config()?;
    let item = plugin
        .get_version(&version)
        .ok_or_else(|| format!("version not found: {sdk}@{version}"))?;
    let req = ins::InstallRequest {
        sdk_name: plugin.sdk_name.clone(),
        plugin_name: plugin.plugin_name.clone(),
        version_name: version.clone(),
        version: item,
        ic,
        mode: ins::InvokeMode::Globally,
        no_envs: false,
    };
    if version == "all" {
        let _ = req;
        finder::uninstall_all(&plugin.sdk_name);
        println!("all versions of {} uninstalled.", plugin.plugin_name);
    } else {
        ins::uninstall(&req)?;
        println!("{} {} uninstalled.", plugin.plugin_name, version);
    }
    Ok(())
}

fn cmd_add_completions(shell: &str) -> Result<(), String> {
    let kind = vmr_completions::shell_kind_from_str(shell)
        .ok_or_else(|| format!("unsupported shell: {shell}"))?;
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let mut buf = Vec::new();
    match kind {
        vmr_completions::ShellKind::Bash => clap_complete::generate(
            clap_complete::shells::Bash,
            &mut cmd,
            name.clone(),
            &mut buf,
        ),
        vmr_completions::ShellKind::Zsh => {
            clap_complete::generate(clap_complete::shells::Zsh, &mut cmd, name.clone(), &mut buf)
        }
        vmr_completions::ShellKind::Fish => clap_complete::generate(
            clap_complete::shells::Fish,
            &mut cmd,
            name.clone(),
            &mut buf,
        ),
        vmr_completions::ShellKind::PowerShell => {
            clap_complete::generate(clap_complete::shells::PowerShell, &mut cmd, name, &mut buf)
        }
    }
    let script = String::from_utf8_lossy(&buf).into_owned();
    vmr_completions::install_completions(kind, &script)?;
    println!("completions installed for {shell}.");
    Ok(())
}

#[allow(dead_code)]
fn _unused_refs(_: &Path) {}
