//! `vmr doctor`: read-only diagnosis of the vmr installation, shell integration,
//! environment variables and `PATH`.
//!
//! The check layer must not change anything: it never creates directories, never loads Lua
//! plugins (which would download the plugin bundle), and never touches the network. Paths
//! therefore come from the side-effect-free accessors in `vmr_core::paths`, and the process
//! environment is snapshotted *before* `VMRConf::new()` writes `conf.toml` values into it, so
//! the report can tell where an effective value came from.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::{Value, json};

use vmr_core::conf::{self, VMRConf};
use vmr_core::{envs, paths};
use vmr_shell::{Kind, Shell};

/// Raw process environment captured before `VMRConf::new()` mutates it.
static RAW_ENV: OnceLock<Env> = OnceLock::new();

pub(crate) fn capture_raw_env() {
    let _ = RAW_ENV.set(Env::capture());
}

fn raw_env() -> Env {
    RAW_ENV.get().cloned().unwrap_or_else(Env::capture)
}

/// Case-insensitive on Windows, exact elsewhere, matching `std::env::var`.
#[derive(Debug, Clone, Default)]
struct Env {
    map: BTreeMap<String, String>,
    non_utf8: Vec<String>,
}

impl Env {
    fn capture() -> Self {
        let mut map = BTreeMap::new();
        let mut non_utf8 = Vec::new();
        for (key, value) in std::env::vars_os() {
            match (key.into_string(), value.into_string()) {
                (Ok(key), Ok(value)) => {
                    map.insert(key, value);
                }
                (Ok(key), Err(_)) => non_utf8.push(key),
                (Err(key), _) => non_utf8.push(key.to_string_lossy().into_owned()),
            }
        }
        Self { map, non_utf8 }
    }

    #[cfg(test)]
    fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            map: pairs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            non_utf8: Vec::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        #[cfg(windows)]
        {
            self.map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v.as_str())
        }
        #[cfg(not(windows))]
        {
            self.map.get(key).map(String::as_str)
        }
    }

    fn non_empty(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|value| !value.is_empty())
    }
}

pub(crate) struct Options {
    pub json: bool,
    pub quiet: bool,
    pub strict: bool,
    pub no_emoji: bool,
    pub shell: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    Warn,
    Fail,
    Skip,
}

impl Status {
    fn json(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Fail => "fail",
            Status::Skip => "skip",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Status::Ok => "✅",
            Status::Warn => "⚠️",
            Status::Fail => "❌",
            Status::Skip => "⏭️",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Warn => "WARN",
            Status::Fail => "FAIL",
            Status::Skip => "SKIP",
        }
    }
}

#[derive(Clone)]
struct Finding {
    id: &'static str,
    status: Status,
    message: String,
    details: Vec<String>,
    fix: Option<String>,
}

impl Finding {
    fn new(id: &'static str, status: Status, message: impl Into<String>) -> Self {
        Self {
            id,
            status,
            message: message.into(),
            details: Vec::new(),
            fix: None,
        }
    }

    fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    fn detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    fn details(mut self, details: impl IntoIterator<Item = String>) -> Self {
        self.details.extend(details);
        self
    }

    fn fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}

struct Section {
    id: &'static str,
    title: &'static str,
    emoji: &'static str,
    findings: Vec<Finding>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Summary {
    ok: usize,
    warn: usize,
    fail: usize,
    skip: usize,
}

impl Summary {
    fn of(sections: &[Section]) -> Self {
        let mut summary = Self::default();
        for finding in sections.iter().flat_map(|section| &section.findings) {
            match finding.status {
                Status::Ok => summary.ok += 1,
                Status::Warn => summary.warn += 1,
                Status::Fail => summary.fail += 1,
                Status::Skip => summary.skip += 1,
            }
        }
        summary
    }
}

/// Paths and values every check reads; read-only by construction.
struct Ctx {
    env: Env,
    conf: VMRConf,
    kind: Kind,
    home: String,
    work_dir: PathBuf,
    conf_path: PathBuf,
    versions_dir: PathBuf,
    rc_path: PathBuf,
    env_file: PathBuf,
    env_keys: PathBuf,
    cd_cache_file: PathBuf,
    plugins_dir: PathBuf,
    shim_dir: PathBuf,
    session: bool,
    platform: String,
}

impl Ctx {
    fn load(conf: VMRConf, opts: &Options) -> Result<Self, String> {
        let env = raw_env();
        let kind = match opts.shell.as_deref() {
            Some(name) => Kind::parse(name).ok_or_else(|| {
                format!("unsupported shell: {name} (expected bash, zsh, fish or powershell)")
            })?,
            None => vmr_shell::detect_kind(),
        };
        let shell = Shell::new(kind);
        let work_dir = paths::work_dir_path();
        let root = if !conf.sdk_installation_dir.is_empty() {
            PathBuf::from(&conf.sdk_installation_dir)
        } else if let Some(dir) = env.non_empty(envs::SDK_INSTALLATION_DIR) {
            PathBuf::from(dir)
        } else {
            work_dir.clone()
        };
        Ok(Self {
            session: env.non_empty(envs::VM_DISABLE).is_some(),
            env,
            conf,
            kind,
            home: shell.home().to_string(),
            conf_path: paths::conf_file_path().unwrap_or_else(paths::conf_file_default_path),
            versions_dir: root.join("versions"),
            rc_path: shell.conf_path(),
            env_file: shell.vm_env_conf_path(),
            env_keys: work_dir.join("env_keys.txt"),
            cd_cache_file: work_dir.join("cd_hook.cache"),
            plugins_dir: work_dir.join("plugins"),
            shim_dir: work_dir.join("bin"),
            work_dir,
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        })
    }

    #[cfg(test)]
    fn for_test(root: &Path, kind: Kind, env: Env, conf: VMRConf) -> Self {
        let work_dir = root.join(".vmr");
        let rc_path = root.join(match kind {
            Kind::Fish => ".config/fish/config.fish",
            Kind::Zsh => ".zshrc",
            Kind::PowerShell => "profile.ps1",
            Kind::Bash => ".bashrc",
        });
        Self {
            env,
            conf,
            kind,
            home: root.to_string_lossy().into_owned(),
            conf_path: work_dir.join("conf.toml"),
            versions_dir: work_dir.join("versions"),
            env_file: work_dir.join(if kind == Kind::Fish {
                "vmr.fish"
            } else {
                "vmr.sh"
            }),
            env_keys: work_dir.join("env_keys.txt"),
            cd_cache_file: work_dir.join("cd_hook.cache"),
            plugins_dir: work_dir.join("plugins"),
            shim_dir: work_dir.join("bin"),
            work_dir,
            session: false,
            platform: "test".to_string(),
            rc_path,
        }
    }
}

/// Prints the report and exits with status 1 when errors (or, under `--strict`, warnings)
/// were found.
pub(crate) fn run(opts: Options, conf: VMRConf, version: &str) -> Result<(), String> {
    let ctx = Ctx::load(conf, &opts)?;
    let sections = diagnose(&ctx);
    let summary = Summary::of(&sections);

    if opts.json {
        let report = report_json(&ctx, version, &sections, summary);
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("failed to serialize the report: {error}"))?
        );
    } else {
        let emoji = !opts.no_emoji && std::io::stdout().is_terminal();
        print_text(&ctx, version, &sections, summary, emoji, opts.quiet);
    }
    let _ = std::io::stdout().flush();

    if summary.fail > 0 || (opts.strict && summary.warn > 0) {
        std::process::exit(1);
    }
    Ok(())
}

fn diagnose(ctx: &Ctx) -> Vec<Section> {
    vec![
        Section {
            id: "environment",
            title: "Environment",
            emoji: "🧩",
            findings: env_checks(ctx),
        },
        Section {
            id: "shell",
            title: "Shell integration",
            emoji: "🐚",
            findings: shell_checks(ctx),
        },
        Section {
            id: "path",
            title: "PATH",
            emoji: "🛤️",
            findings: path_checks(ctx),
        },
        Section {
            id: "versions",
            title: "Versions & links",
            emoji: "📦",
            findings: version_checks(ctx),
        },
        Section {
            id: "session",
            title: "Session",
            emoji: "🔁",
            findings: session_checks(ctx),
        },
    ]
}

// ---------------------------------------------------------------------------
// Environment checks
// ---------------------------------------------------------------------------

fn env_checks(ctx: &Ctx) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.push(check_home(ctx));
    findings.extend(check_conf_parse(ctx));
    findings.push(check_conf_overrides(ctx));
    findings.push(check_proxy(ctx));
    findings.push(check_channel(ctx));
    findings.push(check_token(ctx));
    findings.push(check_threads(ctx));
    findings.push(check_install_dir(ctx));
    findings.push(check_internal_env(ctx));
    if !ctx.env.non_utf8.is_empty() {
        findings.push(
            Finding::new(
                "env.non_utf8",
                Status::Warn,
                "Some environment variables are not valid UTF-8.",
            )
            .details(ctx.env.non_utf8.iter().take(10).cloned())
            .fix("re-export these variables with UTF-8 values"),
        );
    }
    findings
}

fn check_home(ctx: &Ctx) -> Finding {
    #[cfg(windows)]
    let home = ctx.env.get("USERPROFILE").or_else(|| ctx.env.get("HOME"));
    #[cfg(not(windows))]
    let home = ctx.env.get("HOME").or_else(|| ctx.env.get("USERPROFILE"));
    let Some(home) = home else {
        return Finding::new(
            "env.home",
            Status::Warn,
            "Neither HOME nor USERPROFILE is set.",
        )
        .detail(format!("vmr data directory: {}", ctx.work_dir.display()))
        .fix("export HOME=/path/to/your/home");
    };
    if normalize_path_text(home) == normalize_path_text(&ctx.home) {
        Finding::new("env.home", Status::Ok, "Home directory is consistent.")
            .detail(format!("home: {}", ctx.home))
            .detail(format!("vmr data directory: {}", ctx.work_dir.display()))
    } else {
        Finding::new(
            "env.home",
            Status::Warn,
            "The shell home and the vmr home differ.",
        )
        .detail(format!("shell home: {home}"))
        .detail(format!("vmr home:   {}", ctx.home))
        .detail(format!("vmr data directory: {}", ctx.work_dir.display()))
        .fix("check whether HOME/USERPROFILE changed after vmr was configured")
    }
}

fn check_conf_parse(ctx: &Ctx) -> Vec<Finding> {
    if !ctx.conf_path.exists() {
        return vec![
            Finding::new(
                "env.conf",
                Status::Ok,
                "No conf.toml; built-in defaults are in use.",
            )
            .detail(ctx.conf_path.display().to_string()),
        ];
    }
    let mut findings = Vec::new();
    match conf::read_conf_file(&ctx.conf_path) {
        Ok(_) => {
            findings.push(
                Finding::new("env.conf", Status::Ok, "conf.toml parsed successfully.")
                    .detail(ctx.conf_path.display().to_string()),
            );
            if let Ok(keys) = conf::unknown_conf_keys(&ctx.conf_path)
                && !keys.is_empty()
            {
                findings.push(
                    Finding::new(
                        "env.conf_unknown",
                        Status::Warn,
                        "conf.toml contains keys that vmr ignores.",
                    )
                    .detail(format!("unknown keys: {}", keys.join(", ")))
                    .fix(format!(
                        "remove or rename these keys in {}",
                        ctx.conf_path.display()
                    )),
                );
            }
        }
        Err(error) => findings.push(
            Finding::new(
                "env.conf",
                Status::Fail,
                "conf.toml could not be parsed; vmr is using defaults.",
            )
            .details(error.lines().map(str::to_string))
            .fix(format!("fix or remove {}", ctx.conf_path.display())),
        ),
    }
    findings
}

fn check_conf_overrides(ctx: &Ctx) -> Finding {
    let mut conflicts = Vec::new();
    let mut compare = |key: &str, conf_value: Option<String>| {
        let Some(conf_value) = conf_value.filter(|value| !value.is_empty()) else {
            return;
        };
        let Some(shell_value) = ctx.env.non_empty(key) else {
            return;
        };
        if !env_values_equal(key, shell_value, &conf_value) {
            conflicts.push(format!(
                "{key}: shell={shell_value:?} conf={conf_value:?} (conf.toml wins)"
            ));
        }
    };

    compare(envs::LOCAL_PROXY, Some(ctx.conf.proxy_uri.clone()));
    compare(envs::REVERSE_PROXY, Some(ctx.conf.reverse_proxy.clone()));
    compare(
        envs::SDK_INSTALLATION_DIR,
        Some(ctx.conf.sdk_installation_dir.clone()),
    );
    compare(envs::HOST_URL, Some(ctx.conf.version_host_url.clone()));
    if ctx.conf.download_thread_num > 1 {
        compare(
            envs::DOWNLOAD_THREADS,
            Some(ctx.conf.download_thread_num.to_string()),
        );
    }
    if ctx.conf.allow_nested_sessions {
        compare(envs::ALLOW_NESTED_SESSIONS, Some("true".to_string()));
    }

    if conflicts.is_empty() {
        Finding::new(
            "env.conf_override",
            Status::Ok,
            "No shell/conf.toml value conflicts.",
        )
    } else {
        Finding::new(
            "env.conf_override",
            Status::Warn,
            "conf.toml silently overrides shell environment variables.",
        )
        .details(conflicts)
        .fix("unset the shell variable or remove the field from conf.toml")
    }
}

fn env_values_equal(key: &str, shell: &str, conf: &str) -> bool {
    match key {
        envs::HOST_URL => shell.trim_end_matches('/') == conf.trim_end_matches('/'),
        envs::SDK_INSTALLATION_DIR => normalize_path_text(shell) == normalize_path_text(conf),
        envs::DOWNLOAD_THREADS => shell.parse::<i32>().ok() == conf.parse::<i32>().ok(),
        envs::ALLOW_NESTED_SESSIONS => truthy(shell) == truthy(conf),
        _ => shell == conf,
    }
}

fn check_proxy(ctx: &Ctx) -> Finding {
    let local = if ctx.conf.proxy_uri.is_empty() {
        ctx.env.non_empty(envs::LOCAL_PROXY).map(str::to_string)
    } else {
        Some(ctx.conf.proxy_uri.trim().to_string())
    };
    let reverse = if ctx.conf.reverse_proxy.is_empty() {
        ctx.env.non_empty(envs::REVERSE_PROXY).map(str::to_string)
    } else {
        Some(ctx.conf.reverse_proxy.trim().to_string())
    };

    let mut finding = Finding::new(
        "env.proxy",
        Status::Ok,
        "No local or reverse proxy configured.",
    );
    if local.is_none() && reverse.is_none() {
        return finding;
    }

    if let Some(local) = &local {
        finding = finding
            .with_status(Status::Ok)
            .with_message("Proxy configuration detected.");
        if let Some(reverse) = &reverse {
            finding = finding
                .with_status(Status::Warn)
                .with_message("A local proxy is configured; the reverse proxy is ignored.")
                .detail(format!("local proxy: {local}"))
                .detail(format!("reverse proxy: {reverse}"))
                .detail("vmr policy: a non-empty local proxy disables the reverse proxy")
                .fix("unset VMR_REVERSE_PROXY or ProxyUri if the reverse proxy should be used");
        }
        if !has_url_scheme(local) {
            finding = finding
                .with_status(Status::Warn)
                .detail(format!("local proxy has no URL scheme: {local}"));
        }
    }
    if let Some(reverse) = &reverse
        && !has_url_scheme(reverse)
    {
        finding = finding
            .with_status(Status::Warn)
            .detail(format!("reverse proxy has no URL scheme: {reverse}"));
    }
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"] {
        if let Some(value) = ctx.env.non_empty(key) {
            finding = finding.detail(format!("{key} is also set: {value}"));
        }
    }
    finding
}

fn check_channel(ctx: &Ctx) -> Finding {
    let pixi = ctx.env.non_empty(envs::PIXI_CHANNEL);
    let conda = ctx.env.non_empty(envs::CONDA_CHANNEL);
    match (pixi, conda) {
        (Some(pixi), Some(conda)) => Finding::new(
            "env.channel",
            Status::Warn,
            "VMR_PIXI_CHANNEL hides the legacy VMR_CONDA_CHANNEL.",
        )
        .detail(format!("VMR_PIXI_CHANNEL={pixi}"))
        .detail(format!("VMR_CONDA_CHANNEL={conda} (ignored)"))
        .fix("unset VMR_CONDA_CHANNEL"),
        (Some(pixi), None) => {
            let mut finding =
                Finding::new("env.channel", Status::Ok, format!("pixi channel: {pixi}"));
            if !has_url_scheme(pixi) {
                finding = finding
                    .with_status(Status::Warn)
                    .detail("the channel URL has no http(s) scheme")
                    .fix("use a full channel URL, e.g. https://conda.anaconda.org/conda-forge");
            }
            finding
        }
        (None, Some(conda)) => Finding::new(
            "env.channel",
            Status::Warn,
            "VMR_CONDA_CHANNEL is a legacy name.",
        )
        .detail(format!("legacy value: {conda}"))
        .fix("export VMR_PIXI_CHANNEL=<same value>; unset VMR_CONDA_CHANNEL"),
        (None, None) => Finding::new(
            "env.channel",
            Status::Ok,
            format!(
                "pixi channel: default {}",
                vmr_pixi::repodata::DEFAULT_CHANNEL
            ),
        ),
    }
}

fn check_token(ctx: &Ctx) -> Finding {
    let mut sources: Vec<(&str, String)> = Vec::new();
    if !ctx.conf.github_token.trim().is_empty() {
        sources.push((
            "conf.toml GithubToken",
            ctx.conf.github_token.trim().to_string(),
        ));
    }
    if let Some(token) = ctx.env.non_empty(envs::VMR_GITHUB_TOKEN) {
        sources.push((envs::VMR_GITHUB_TOKEN, token.trim().to_string()));
    }
    if let Some(token) = ctx.env.non_empty(envs::GITHUB_TOKEN) {
        sources.push((envs::GITHUB_TOKEN, token.trim().to_string()));
    }

    if sources.is_empty() {
        return Finding::new(
            "env.github_token",
            Status::Ok,
            "No GitHub token configured; API requests are anonymous (60/hour).",
        )
        .detail("set VMR_GITHUB_TOKEN or conf.toml GithubToken to raise the API limit");
    }

    let distinct: BTreeSet<&str> = sources.iter().map(|(_, token)| token.as_str()).collect();
    if distinct.len() > 1 {
        Finding::new(
            "env.github_token",
            Status::Warn,
            "Multiple different GitHub tokens are configured; only one is used.",
        )
        .details(
            sources
                .iter()
                .map(|(source, token)| format!("{source}: {}", redact(token))),
        )
        .detail("precedence: conf.toml GithubToken > VMR_GITHUB_TOKEN > GITHUB_TOKEN")
        .fix("keep a single token and unset the others")
    } else {
        let (source, token) = &sources[0];
        Finding::new("env.github_token", Status::Ok, "GitHub token configured.")
            .detail(format!("{source}: {}", redact(token)))
    }
}

fn check_threads(ctx: &Ctx) -> Finding {
    let raw = ctx.env.non_empty(envs::DOWNLOAD_THREADS);
    let conf_threads = ctx.conf.download_thread_num;
    let effective = if conf_threads > 1 {
        conf_threads
    } else {
        raw.and_then(|value| value.parse::<i32>().ok())
            .filter(|value| *value >= 1)
            .unwrap_or(1)
    };
    let mut finding = Finding::new(
        "env.threads",
        Status::Ok,
        format!("Download threads: {effective}"),
    );
    if let Some(raw) = raw {
        match raw.parse::<i32>() {
            Ok(value) if value >= 1 => {}
            _ => {
                finding = finding
                    .with_status(Status::Warn)
                    .with_message(format!(
                        "VMR_DOWNLOAD_THREADS={raw:?} is invalid; vmr uses {effective}."
                    ))
                    .fix("export VMR_DOWNLOAD_THREADS=<positive integer>");
            }
        }
    }
    if conf_threads < 0 {
        finding = finding
            .with_status(Status::Warn)
            .detail(format!("conf.toml ThreadNum={conf_threads} is negative"))
            .fix("set ThreadNum to 1 or more in conf.toml");
    }
    finding
}

fn check_install_dir(ctx: &Ctx) -> Finding {
    let value = if !ctx.conf.sdk_installation_dir.is_empty() {
        Some(ctx.conf.sdk_installation_dir.clone())
    } else {
        ctx.env
            .non_empty(envs::SDK_INSTALLATION_DIR)
            .map(str::to_string)
    };
    let Some(value) = value else {
        return Finding::new(
            "env.install_dir",
            Status::Ok,
            "Version directory: default location.",
        )
        .detail(format!("versions: {}", ctx.versions_dir.display()))
        .detail(format!(
            "cache:    {}",
            ctx.versions_dir
                .parent()
                .unwrap_or(&ctx.work_dir)
                .join("cache")
                .display()
        ));
    };

    let path = Path::new(&value);
    let mut finding = Finding::new(
        "env.install_dir",
        Status::Ok,
        format!("Version directory: {value}"),
    )
    .detail(format!("versions: {}", ctx.versions_dir.display()));

    if value.starts_with('~') {
        return finding
            .with_status(Status::Fail)
            .with_message("VMR_SDK_INSTALLATION_DIR starts with '~', which vmr does not expand.")
            .fix("use an absolute path");
    }
    if path.is_relative() {
        return finding
            .with_status(Status::Fail)
            .with_message("VMR_SDK_INSTALLATION_DIR is a relative path.")
            .fix("export VMR_SDK_INSTALLATION_DIR=/absolute/path");
    }
    if path.exists() && !path.is_dir() {
        return finding
            .with_status(Status::Fail)
            .with_message("VMR_SDK_INSTALLATION_DIR exists but is not a directory.")
            .detail(value)
            .fix("point the variable at a directory");
    }
    if path.exists() {
        if writable(path) == Some(false) {
            finding = finding
                .with_status(Status::Fail)
                .with_message("VMR_SDK_INSTALLATION_DIR is not writable.")
                .detail(value)
                .fix("fix the directory permissions or choose another path");
        }
    } else if let Some(ancestor) = nearest_existing(path) {
        if writable(&ancestor) == Some(false) {
            finding = finding
                .with_status(Status::Fail)
                .with_message("The parent of VMR_SDK_INSTALLATION_DIR is not writable.")
                .detail(format!(
                    "nearest existing directory: {}",
                    ancestor.display()
                ))
                .fix("fix the directory permissions or choose another path");
        } else {
            finding = finding.detail("the directory will be created on first install");
        }
    }
    finding
}

fn check_internal_env(ctx: &Ctx) -> Finding {
    let mut details = Vec::new();
    if ctx.env.non_empty(envs::MCP_SERVER).is_some() {
        details.push(format!(
            "{} is set; this variable is reserved for MCP server child processes",
            envs::MCP_SERVER
        ));
    }
    for key in [envs::GVC_DEFAULT_PROXY, envs::VCOLLECTOR_PROXY] {
        if ctx.env.non_empty(key).is_some() {
            details.push(format!("{key} is set but no longer read by vmr"));
        }
    }
    if ctx.env.non_empty(envs::ADD_TO_PATH_TEMPORARILY).is_some() && !ctx.session {
        details.push(format!(
            "{} is set outside a session shell",
            envs::ADD_TO_PATH_TEMPORARILY
        ));
    }
    if let Some(value) = ctx.env.non_empty(envs::COURSIER_PATH) {
        details.push(format!(
            "{}={value} (used by the coursier installer)",
            envs::COURSIER_PATH
        ));
    }

    if details.is_empty() {
        Finding::new(
            "env.internal",
            Status::Ok,
            "No internal or legacy variables set.",
        )
    } else if details.iter().any(|detail| {
        detail.contains("no longer read")
            || detail.contains("reserved")
            || detail.contains("outside a session")
    }) {
        Finding::new(
            "env.internal",
            Status::Warn,
            "Internal or legacy environment variables are present.",
        )
        .details(details)
        .fix("unset these variables in the shell configuration")
    } else {
        Finding::new(
            "env.internal",
            Status::Ok,
            "Auxiliary variables are present.",
        )
        .details(details)
    }
}

// ---------------------------------------------------------------------------
// Shell integration checks
// ---------------------------------------------------------------------------

fn shell_checks(ctx: &Ctx) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(check_init_line(ctx));
    #[cfg(not(windows))]
    findings.extend(check_env_file(ctx));
    #[cfg(windows)]
    findings.push(check_windows_env(ctx));
    findings.push(check_env_keys(ctx));
    findings.push(check_cd_hook(ctx));
    findings
}

fn check_init_line(ctx: &Ctx) -> Vec<Finding> {
    let expected = Shell::new(ctx.kind).init_line();
    let data = match fs::read_to_string(&ctx.rc_path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return vec![
                Finding::new(
                    "shell.init_line",
                    Status::Fail,
                    format!("Shell integration file not found for {}.", ctx.kind.name()),
                )
                .detail(ctx.rc_path.display().to_string())
                .detail(format!("expected: {expected}"))
                .fix(append_fix(ctx.kind, &ctx.rc_path, &expected)),
            ];
        }
        Err(error) => {
            return vec![
                Finding::new(
                    "shell.init_line",
                    Status::Fail,
                    "Shell integration file could not be read.",
                )
                .detail(ctx.rc_path.display().to_string())
                .detail(error.to_string()),
            ];
        }
    };

    let mut count = 0usize;
    let mut wrong = Vec::new();
    let mut legacy = Vec::new();
    for line in data.lines() {
        let line = line.trim();
        if line == expected {
            count += 1;
        } else if line.contains("vmr init") {
            wrong.push(line.to_string());
        }
        if line.starts_with("# vm_envs start")
            || line.starts_with("# vmr source alias start")
            || line.starts_with("# VMR Completions (")
        {
            legacy.push(line.to_string());
        }
    }

    let mut findings = Vec::new();
    match count {
        0 => {
            let mut finding = Finding::new(
                "shell.init_line",
                Status::Fail,
                format!("Shell integration is not enabled for {}.", ctx.kind.name()),
            )
            .detail(ctx.rc_path.display().to_string())
            .detail(format!("expected: {expected}"))
            .fix(append_fix(ctx.kind, &ctx.rc_path, &expected));
            if !wrong.is_empty() {
                finding = finding.detail(format!("found instead: {}", wrong.join(" | ")));
            }
            if let Some(other) = other_enabled_shell(ctx.kind) {
                finding = finding.detail(format!(
                    "integration for another shell is active: {}",
                    other.display()
                ));
            }
            findings.push(finding);
        }
        1 => findings.push(
            Finding::new(
                "shell.init_line",
                Status::Ok,
                format!("Shell integration enabled for {}.", ctx.kind.name()),
            )
            .detail(ctx.rc_path.display().to_string()),
        ),
        _ => findings.push(
            Finding::new(
                "shell.init_line",
                Status::Warn,
                format!("Shell integration line is duplicated ({count} times)."),
            )
            .detail(ctx.rc_path.display().to_string())
            .detail(format!("line: {expected}"))
            .fix("remove the duplicate lines, keeping one"),
        ),
    }

    if !legacy.is_empty() {
        findings.push(
            Finding::new(
                "shell.legacy",
                Status::Warn,
                "Legacy vmr blocks are still present in the shell configuration.",
            )
            .details(legacy)
            .fix("remove the marked legacy blocks; the next `vmr use` migrates them"),
        );
    }
    findings
}

fn other_enabled_shell(current: Kind) -> Option<PathBuf> {
    [Kind::Bash, Kind::Zsh, Kind::Fish, Kind::PowerShell]
        .into_iter()
        .filter(|kind| *kind != current)
        .find_map(|kind| {
            let shell = Shell::new(kind);
            let path = shell.conf_path();
            let line = shell.init_line();
            fs::read_to_string(&path)
                .ok()
                .filter(|data| {
                    data.lines()
                        .any(|candidate| candidate.trim() == line.as_str())
                })
                .map(|_| path)
        })
}

fn append_fix(kind: Kind, path: &Path, line: &str) -> String {
    match kind {
        Kind::Fish => {
            let parent = path.parent().unwrap_or(Path::new("."));
            format!(
                "mkdir -p \"{}\" && echo '{}' >> \"{}\"",
                parent.display(),
                line,
                path.display()
            )
        }
        Kind::PowerShell => format!("Add-Content -Path \"{}\" -Value '{}'", path.display(), line),
        _ => format!("echo '{}' >> \"{}\"", line, path.display()),
    }
}

#[cfg(not(windows))]
fn check_env_file(ctx: &Ctx) -> Vec<Finding> {
    if !ctx.env_file.exists() {
        if has_active_sdk(ctx) {
            return vec![
                Finding::new(
                    "shell.env_file",
                    Status::Fail,
                    "The environment file is missing while an SDK is active.",
                )
                .detail(ctx.env_file.display().to_string())
                .detail("active SDK links exist, but their PATH entries were never written")
                .fix("run `vmr use <sdk>@<version>` for the active SDK to regenerate it"),
            ];
        }
        return vec![
            Finding::new(
                "shell.env_file",
                Status::Ok,
                "Environment file not created yet (no active SDK).",
            )
            .detail(ctx.env_file.display().to_string()),
        ];
    }

    let data = match fs::read_to_string(&ctx.env_file) {
        Ok(data) => data,
        Err(error) => {
            return vec![
                Finding::new(
                    "shell.env_file",
                    Status::Fail,
                    "The environment file could not be read.",
                )
                .detail(ctx.env_file.display().to_string())
                .detail(error.to_string()),
            ];
        }
    };

    let mut paths: Vec<String> = Vec::new();
    let mut vars: Vec<String> = Vec::new();
    let mut foreign = Vec::new();
    let mut unknown = Vec::new();
    let mut line_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (index, raw) in data.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        *line_counts.entry(line.to_string()).or_default() += 1;
        match parse_env_line(ctx.kind, line) {
            EnvLine::Path(path) => {
                if ctx.kind == Kind::Fish {
                    paths.push(expand_home(&path, &ctx.home));
                } else {
                    // A single `export PATH="a:b"` line contributes two PATH entries.
                    paths.extend(
                        path.split(':')
                            .filter(|part| !part.is_empty())
                            .map(|part| expand_home(part, &ctx.home)),
                    );
                }
            }
            EnvLine::Var(key) => vars.push(key),
            EnvLine::Unknown => {
                if foreign_syntax(ctx.kind, line) {
                    foreign.push(format!("line {}: {line}", index + 1));
                } else {
                    unknown.push(format!("line {}: {line}", index + 1));
                }
            }
        }
    }

    let mut findings = Vec::new();
    if !foreign.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_syntax",
                Status::Fail,
                "The environment file uses syntax for a different shell.",
            )
            .details(cap(foreign, 5))
            .detail(ctx.env_file.display().to_string())
            .fix("remove the file and run `vmr use <sdk>@<version>` to regenerate it"),
        );
    }

    let mut dead_vmr = Vec::new();
    let mut dead_other = Vec::new();
    let mut duplicates = Vec::new();
    let mut path_counts: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in &paths {
        let normalized = normalize_path_text(path);
        path_counts
            .entry(normalized)
            .or_default()
            .push(path.clone());
        if !Path::new(path).exists() {
            if managed_entry(ctx, path) {
                dead_vmr.push(path.clone());
            } else {
                dead_other.push(path.clone());
            }
        }
    }
    for (_, entries) in path_counts {
        if entries.len() > 1 {
            duplicates.push(format!("{} ({} times)", entries[0], entries.len()));
        }
    }
    for (line, count) in &line_counts {
        if *count > 1 && !duplicates.iter().any(|entry| entry.contains(line)) {
            duplicates.push(format!("{line} ({count} times)"));
        }
    }

    if !dead_vmr.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_dead",
                Status::Fail,
                "The environment file points at directories that no longer exist.",
            )
            .details(cap(dead_vmr, 8))
            .fix("run `vmr use` for the affected SDK or remove the stale lines"),
        );
    }
    if !dead_other.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_dead_other",
                Status::Warn,
                "The environment file contains paths outside the vmr directory that are missing.",
            )
            .details(cap(dead_other, 8))
            .fix("remove or fix these entries"),
        );
    }
    if !duplicates.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_duplicate",
                Status::Warn,
                "The environment file contains duplicates.",
            )
            .details(cap(duplicates, 8))
            .fix("remove duplicate lines and PATH entries"),
        );
    }
    if !unknown.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_unknown",
                Status::Warn,
                "The environment file contains unrecognized lines.",
            )
            .details(cap(unknown, 8))
            .fix("remove these lines or regenerate the file with `vmr use`"),
        );
    }
    if findings.is_empty() {
        findings.push(
            Finding::new(
                "shell.env_file",
                Status::Ok,
                format!(
                    "Environment file: {} PATH {}, {} {}.",
                    paths.len(),
                    if paths.len() == 1 { "entry" } else { "entries" },
                    vars.len(),
                    if vars.len() == 1 {
                        "variable"
                    } else {
                        "variables"
                    }
                ),
            )
            .detail(ctx.env_file.display().to_string()),
        );
    }
    findings
}

#[cfg(windows)]
fn check_windows_env(ctx: &Ctx) -> Finding {
    let path = vmr_shell::user_env("PATH").unwrap_or_default();
    let versions = vmr_shell::user_env(envs::VERSIONS);
    let mut details = Vec::new();
    let mut status = Status::Ok;
    let shim = ctx.shim_dir.to_string_lossy().into_owned();
    let has_shim = path
        .split(';')
        .any(|entry| normalize_path_text(entry) == normalize_path_text(&shim));
    if ctx.shim_dir.join("shims.json").exists() && !has_shim {
        status = Status::Fail;
        details.push(format!("{shim} is not in the user PATH"));
    }
    if let Some(versions) = &versions {
        if normalize_path_text(versions) != normalize_path_text(&ctx.versions_dir.to_string_lossy())
        {
            status = Status::Warn;
            details.push(format!(
                "{}={versions} (expected {})",
                envs::VERSIONS,
                ctx.versions_dir.display()
            ));
        }
    }
    if status == Status::Ok {
        Finding::new(
            "shell.env_file",
            Status::Ok,
            "Windows user environment is consistent.",
        )
        .detail(format!("shim directory: {shim}"))
    } else {
        Finding::new(
            "shell.env_file",
            status,
            "The Windows user environment is not consistent.",
        )
        .details(details)
        .fix("run a global `vmr use` to rewrite HKCU\\Environment")
    }
}

fn check_env_keys(ctx: &Ctx) -> Finding {
    let mut details = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let data = fs::read_to_string(&ctx.env_keys).ok();
    if let Some(data) = &data {
        for (index, raw) in data.lines().enumerate() {
            let name = raw.trim();
            if name.is_empty() {
                details.push(format!("line {} is empty", index + 1));
                continue;
            }
            if name.eq_ignore_ascii_case("PATH") {
                details.push("PATH must not be listed in env_keys.txt".to_string());
                continue;
            }
            if !valid_env_name(name) {
                details.push(format!("invalid variable name: {name}"));
            }
            if names.iter().any(|existing| existing == name) {
                details.push(format!("duplicate variable: {name}"));
            }
            names.push(name.to_string());
        }
    }

    #[cfg(not(windows))]
    let vars = env_file_vars(ctx);
    #[cfg(windows)]
    let vars: Vec<String> = Vec::new();
    let missing: Vec<String> = vars
        .into_iter()
        .filter(|key| !names.iter().any(|name| name == key))
        .collect();
    if !missing.is_empty() {
        if data.is_none() {
            details.push(
                "the environment file defines variables but env_keys.txt is missing".to_string(),
            );
        } else {
            details.push(format!(
                "variables in the environment file are not recorded: {}",
                missing.join(", ")
            ));
        }
    }

    if details.is_empty() {
        Finding::new("shell.env_keys", Status::Ok, "env_keys.txt is consistent.")
    } else {
        Finding::new(
            "shell.env_keys",
            Status::Warn,
            "env_keys.txt needs attention.",
        )
        .details(cap(details, 8))
        .fix("remove invalid or duplicate entries; it is rewritten by the next global `vmr use`")
    }
}

fn check_cd_hook(ctx: &Ctx) -> Finding {
    let init_set = ctx.env.non_empty(envs::CD_INIT).is_some();
    let init_enabled = fs::read_to_string(&ctx.rc_path)
        .ok()
        .map(|data| {
            let expected = Shell::new(ctx.kind).init_line();
            data.lines().any(|line| line.trim() == expected)
        })
        .unwrap_or(false);
    if !init_enabled && !init_set {
        return Finding::new(
            "shell.cd_hook",
            Status::Skip,
            "Shell integration is not enabled; the cd hook is inactive.",
        );
    }

    let cache_value = ctx.env.non_empty(envs::CD_CACHE).map(str::to_string);
    let file_value = fs::read_to_string(&ctx.cd_cache_file)
        .ok()
        .and_then(|data| data.lines().next().map(str::trim).map(str::to_string))
        .filter(|value| !value.is_empty());
    let mut details = Vec::new();
    let mut warn = false;
    let mut needs_reload = false;

    if !init_set && init_enabled {
        warn = true;
        needs_reload = true;
        details.push(
            "VMR_CD_INIT is not set; the shell integration has not been evaluated here".to_string(),
        );
    }
    if let Some(cache) = &cache_value {
        match &file_value {
            None => {
                warn = true;
                details.push(format!(
                    "VMR_CD_CACHE is set, but {} is empty or missing",
                    ctx.cd_cache_file.display()
                ));
            }
            Some(value) if normalize_path_text(value) != normalize_path_text(cache) => {
                warn = true;
                details.push(format!(
                    "VMR_CD_CACHE={cache} does not match the cache file value {value}"
                ));
            }
            Some(_) => {}
        }
        let dir = Path::new(cache);
        if !dir.exists() {
            warn = true;
            details.push(format!("cached project directory is gone: {cache}"));
        } else {
            if !dir.join(".vmr.lock").exists() {
                warn = true;
                details.push(format!("cached project has no .vmr.lock: {cache}"));
            }
            if let Ok(current) = std::env::current_dir()
                && !path_under(&current, dir)
            {
                details.push(format!(
                    "current directory {} is outside the cached project",
                    current.display()
                ));
            }
        }
    } else if file_value.is_some() {
        warn = true;
        details.push(format!(
            "{} exists but VMR_CD_CACHE is unset",
            ctx.cd_cache_file.display()
        ));
    }

    if warn {
        let fix = if needs_reload {
            match ctx.kind {
                Kind::PowerShell => ". $PROFILE".to_string(),
                _ => format!("exec {}", ctx.kind.name()),
            }
        } else {
            match ctx.kind {
                Kind::PowerShell => "vmrsource".to_string(),
                _ => "cd .".to_string(),
            }
        };
        Finding::new(
            "shell.cd_hook",
            Status::Warn,
            "cd hook state is not consistent.",
        )
        .with_status(Status::Warn)
        .details(details)
        .fix(fix)
    } else {
        Finding::new("shell.cd_hook", Status::Ok, "cd hook state is consistent.")
    }
}

// ---------------------------------------------------------------------------
// PATH checks
// ---------------------------------------------------------------------------

fn path_checks(ctx: &Ctx) -> Vec<Finding> {
    let entries = path_entries(ctx);
    let mut findings = vec![
        check_path_duplicates(ctx, &entries),
        check_path_missing(ctx, &entries),
        check_path_empty(&entries),
        check_vmr_binary(&entries),
    ];
    findings.extend(check_active_commands(ctx, &entries));
    findings.push(check_shim_dir(ctx, &entries));
    findings
}

fn path_entries(ctx: &Ctx) -> Vec<String> {
    let separator = if cfg!(windows) { ';' } else { ':' };
    ctx.env
        .get("PATH")
        .unwrap_or_default()
        .split(separator)
        .map(str::to_string)
        .collect()
}

fn check_path_duplicates(ctx: &Ctx, entries: &[String]) -> Finding {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in entries {
        if entry.is_empty() || !managed_entry(ctx, entry) {
            continue;
        }
        seen.entry(normalize_path_text(entry))
            .or_default()
            .push(entry.clone());
    }
    let duplicates: Vec<String> = seen
        .into_values()
        .filter(|values| values.len() > 1)
        .map(|values| format!("{} ({} times)", values[0], values.len()))
        .collect();
    if duplicates.is_empty() {
        Finding::new(
            "path.duplicate",
            Status::Ok,
            "No duplicate vmr PATH entries.",
        )
    } else {
        Finding::new(
            "path.duplicate",
            Status::Warn,
            "PATH contains duplicate vmr entries.",
        )
        .details(cap(duplicates, 10))
        .fix("remove the duplicates; `_vmr_refresh` keeps new ones from stacking")
    }
}

fn check_path_missing(ctx: &Ctx, entries: &[String]) -> Finding {
    let mut missing = Vec::new();
    for entry in entries {
        if entry.is_empty() || !managed_entry(ctx, entry) {
            continue;
        }
        let path = Path::new(entry);
        if path == ctx.work_dir || path == ctx.versions_dir {
            continue;
        }
        if !path.exists() {
            missing.push(entry.clone());
        }
    }
    if missing.is_empty() {
        Finding::new("path.stale", Status::Ok, "No stale vmr PATH entries.")
    } else {
        Finding::new(
            "path.stale",
            Status::Fail,
            "PATH points at vmr directories that no longer exist.",
        )
        .details(cap(missing, 10))
        .fix("remove the stale entries or reinstall the affected SDK")
    }
}

fn check_path_empty(entries: &[String]) -> Finding {
    let empty = entries.iter().filter(|entry| entry.is_empty()).count();
    if empty == 0 {
        Finding::new("path.empty", Status::Ok, "No empty PATH entries.")
    } else {
        Finding::new(
            "path.empty",
            Status::Warn,
            format!(
                "PATH contains {empty} empty entr{}; this adds the current directory.",
                if empty == 1 { "y" } else { "ies" }
            ),
        )
        .fix("remove `::` (or a trailing separator) from PATH")
    }
}

fn check_vmr_binary(entries: &[String]) -> Finding {
    let name = format!("vmr{}", std::env::consts::EXE_SUFFIX);
    let found = entries
        .iter()
        .filter(|entry| !entry.is_empty())
        .find_map(|entry| {
            let path = Path::new(entry).join(&name);
            if path.is_file() { Some(path) } else { None }
        });
    let current = std::env::current_exe().ok();

    match found {
        None => {
            let mut finding = Finding::new(
                "path.vmr_binary",
                Status::Warn,
                "The vmr executable is not on PATH.",
            )
            .fix("add the directory containing vmr to PATH");
            if let Some(current) = current {
                finding = finding.detail(format!("running: {}", current.display()));
            }
            finding
        }
        Some(found) => {
            // Windows package managers install launcher shims (Scoop) whose path differs
            // from the real executable even though they are correct; only report presence.
            #[cfg(windows)]
            {
                let _ = current;
                Finding::new("path.vmr_binary", Status::Ok, "vmr is on PATH.")
                    .detail(found.display().to_string())
            }
            #[cfg(not(windows))]
            if let Some(current) = current {
                if same_file(&found, &current) {
                    Finding::new(
                        "path.vmr_binary",
                        Status::Ok,
                        "vmr resolves to this binary.",
                    )
                    .detail(found.display().to_string())
                } else {
                    Finding::new(
                        "path.vmr_binary",
                        Status::Warn,
                        "PATH resolves vmr to a different binary.",
                    )
                    .detail(format!("PATH:    {}", found.display()))
                    .detail(format!("running: {}", current.display()))
                    .fix("remove the stale binary or move the intended one earlier in PATH")
                }
            } else {
                Finding::new("path.vmr_binary", Status::Ok, "vmr is on PATH.")
                    .detail(found.display().to_string())
            }
        }
    }
}

#[cfg(not(windows))]
fn check_active_commands(ctx: &Ctx, entries: &[String]) -> Vec<Finding> {
    let scans = scan_sdks(ctx);
    let active_bins: Vec<PathBuf> = scans
        .iter()
        .filter(|scan| !scan.broken)
        .filter_map(|scan| scan.active.as_ref().map(|target| target.join("bin")))
        .collect();
    let mut findings = Vec::new();
    let mut checked = 0usize;
    let mut not_on_path = Vec::new();
    let mut shadowed = Vec::new();
    let mut stale = Vec::new();
    let mut shared = Vec::new();
    let mut unresolved = Vec::new();

    for scan in &scans {
        let (Some(target), false) = (&scan.active, scan.broken) else {
            continue;
        };
        let bin = target.join("bin");
        if !bin.is_dir() {
            continue;
        }
        if !entries
            .iter()
            .any(|entry| !entry.is_empty() && same_dir(Path::new(entry), &bin))
        {
            not_on_path.push(format!("{}: {}", scan.sdk, bin.display()));
            continue;
        }
        let Ok(read_dir) = fs::read_dir(&bin) else {
            continue;
        };
        let mut executables: Vec<PathBuf> = read_dir
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_executable_file(path))
            .collect();
        executables.sort();
        for path in executables.into_iter().take(64) {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            checked += 1;
            match resolve_command(name, entries) {
                Some(found) => {
                    let found_dir = found.parent().unwrap_or(Path::new("."));
                    if same_dir(found_dir, &bin) {
                        continue;
                    }
                    if under_canonical(found_dir, &ctx.versions_dir) {
                        // Another vmr-managed prefix already provides this command. Multiple
                        // active SDKs (especially conda environments) routinely ship the same
                        // helper tools; the most recent `vmr use` wins by PATH order.
                        if active_bins.iter().any(|active| same_dir(active, found_dir)) {
                            shared.push(format!("{name}: provided by {}", found_dir.display()));
                        } else {
                            stale.push(format!(
                                "{name}: {} (inactive vmr version)",
                                found.display()
                            ));
                        }
                    } else {
                        shadowed.push(format!(
                            "{name}: {} (expected under {})",
                            found.display(),
                            bin.display()
                        ));
                    }
                }
                None => unresolved.push(format!("{name}: not found on PATH")),
            }
        }
    }

    if !not_on_path.is_empty() {
        findings.push(
            Finding::new(
                "path.active_bin",
                Status::Fail,
                "The active SDK bin directory is not on PATH.",
            )
            .details(cap(not_on_path, 8))
            .fix("run `vmr use <sdk>@<version>` again to refresh the environment"),
        );
    }
    if !shadowed.is_empty() {
        findings.push(
            Finding::new(
                "path.shadow",
                Status::Fail,
                "Commands resolve outside the active vmr version.",
            )
            .details(cap(shadowed, 8))
            .fix("move the vmr PATH entries before the conflicting toolchain, or remove the conflict"),
        );
    }
    if !stale.is_empty() {
        findings.push(
            Finding::new(
                "path.shadow_inactive",
                Status::Warn,
                "Some commands resolve to vmr versions that are no longer active.",
            )
            .details(cap(stale, 8))
            .fix("reload the shell or run `vmr use` again for the affected SDK"),
        );
    }
    if !shared.is_empty() {
        findings.push(
            Finding::new(
                "path.shared",
                Status::Ok,
                format!(
                    "{} {} provided by more than one active SDK; PATH order decides.",
                    shared.len(),
                    if shared.len() == 1 {
                        "command is"
                    } else {
                        "commands are"
                    }
                ),
            )
            .details(cap(shared, 3))
            .detail(
                "PATH order follows the line order in the env file; re-adding an SDK entry \
                 moves it to the front",
            ),
        );
    }
    if !unresolved.is_empty() {
        findings.push(
            Finding::new(
                "path.unresolved",
                Status::Warn,
                "Some commands of the active version are not found on PATH.",
            )
            .details(cap(unresolved, 8))
            .fix("reload the shell or run `vmr use` again for the active SDK"),
        );
    }
    if findings.is_empty() {
        if checked == 0 {
            findings.push(Finding::new(
                "path.active",
                Status::Ok,
                "No active SDK bin directory to verify.",
            ));
        } else {
            findings.push(Finding::new(
                "path.active",
                Status::Ok,
                format!("Active SDK commands resolve through vmr ({checked} checked)."),
            ));
        }
    }
    findings
}

#[cfg(windows)]
fn check_active_commands(ctx: &Ctx, entries: &[String]) -> Vec<Finding> {
    let state = ctx.shim_dir.join("shims.json");
    let Ok(data) = fs::read_to_string(&state) else {
        return vec![Finding::new(
            "path.active",
            Status::Ok,
            "No Windows shims installed yet.",
        )];
    };
    let Ok(document) = serde_json::from_str::<Value>(&data) else {
        return vec![
            Finding::new(
                "path.active",
                Status::Fail,
                "shims.json could not be parsed.",
            )
            .detail(state.display().to_string())
            .fix("run a global `vmr use` to regenerate the shims"),
        ];
    };
    let commands = document
        .get("commands")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if commands.is_empty() {
        return vec![Finding::new(
            "path.active",
            Status::Ok,
            "No Windows shims installed yet.",
        )];
    }
    if !entries
        .iter()
        .any(|entry| !entry.is_empty() && same_dir(Path::new(entry), &ctx.shim_dir))
    {
        return vec![
            Finding::new(
                "path.active",
                Status::Fail,
                "The Windows shim directory is not on PATH.",
            )
            .detail(ctx.shim_dir.display().to_string())
            .fix("run a global `vmr use` to rewrite HKCU\\Environment"),
        ];
    }
    let total = commands.len();
    let mut missing = Vec::new();
    let mut bad_targets = Vec::new();
    for (name, entry) in commands {
        if !ctx.shim_dir.join(&name).exists() {
            missing.push(name.clone());
        }
        if let Some(target) = entry.get("target").and_then(Value::as_str) {
            if !Path::new(target).exists() {
                bad_targets.push(format!("{name} -> {target}"));
            }
        }
    }
    let mut findings = Vec::new();
    if !missing.is_empty() {
        findings.push(
            Finding::new(
                "path.active_bin",
                Status::Fail,
                "Some Windows shims are missing.",
            )
            .details(cap(missing, 8))
            .fix("run a global `vmr use` to regenerate the shims"),
        );
    }
    if !bad_targets.is_empty() {
        findings.push(
            Finding::new(
                "path.shadow",
                Status::Fail,
                "Some Windows shims point at missing executables.",
            )
            .details(cap(bad_targets, 8))
            .fix("reinstall the affected SDK"),
        );
    }
    if findings.is_empty() {
        findings.push(Finding::new(
            "path.active",
            Status::Ok,
            format!("Windows shims resolve ({total} checked)."),
        ));
    }
    findings
}

fn check_shim_dir(ctx: &Ctx, entries: &[String]) -> Finding {
    let on_path = entries
        .iter()
        .any(|entry| !entry.is_empty() && same_dir(Path::new(entry), &ctx.shim_dir));
    if cfg!(windows) {
        if ctx.shim_dir.join("shims.json").exists() && !on_path {
            Finding::new(
                "path.shim_dir",
                Status::Fail,
                "The Windows shim directory is not on PATH.",
            )
            .detail(ctx.shim_dir.display().to_string())
            .fix("run a global `vmr use` to rewrite HKCU\\Environment")
        } else {
            Finding::new(
                "path.shim_dir",
                Status::Ok,
                "Windows shim directory is consistent.",
            )
        }
    } else if on_path {
        Finding::new(
            "path.shim_dir",
            Status::Warn,
            "The Windows shim directory is on a Unix PATH.",
        )
        .detail(ctx.shim_dir.display().to_string())
        .fix("remove it from PATH; Unix uses the version bin directories directly")
    } else {
        Finding::new(
            "path.shim_dir",
            Status::Ok,
            "No Windows shim directory on PATH.",
        )
    }
}

// ---------------------------------------------------------------------------
// Version disk checks
// ---------------------------------------------------------------------------

struct SdkScan {
    sdk: String,
    root: PathBuf,
    versions: usize,
    link: PathBuf,
    active: Option<PathBuf>,
    active_raw: Option<String>,
    link_error: Option<String>,
    broken: bool,
}

fn scan_sdks(ctx: &Ctx) -> Vec<SdkScan> {
    let Ok(read_dir) = fs::read_dir(&ctx.versions_dir) else {
        return Vec::new();
    };
    let mut entries: Vec<_> = read_dir.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    let mut scans = Vec::new();
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(sdk) = name.strip_suffix("_versions") else {
            continue;
        };
        if sdk.is_empty() || !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let root = entry.path();
        let mut versions = 0usize;
        if let Ok(iter) = fs::read_dir(&root) {
            for item in iter.filter_map(Result::ok) {
                if item.file_name().to_string_lossy() == sdk {
                    continue;
                }
                if item
                    .file_type()
                    .map(|kind| kind.is_dir() && !kind.is_symlink())
                    .unwrap_or(false)
                {
                    versions += 1;
                }
            }
        }
        let link = root.join(sdk);
        let mut scan = SdkScan {
            sdk: sdk.to_string(),
            root: root.clone(),
            versions,
            link,
            active: None,
            active_raw: None,
            link_error: None,
            broken: false,
        };
        match fs::read_link(&scan.link) {
            Ok(raw) => {
                let target = if raw.is_absolute() {
                    raw.clone()
                } else {
                    root.join(&raw)
                };
                scan.broken = !target.exists();
                scan.active = Some(target);
                scan.active_raw = Some(raw.to_string_lossy().into_owned());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => scan.link_error = Some(error.to_string()),
        }
        scans.push(scan);
    }
    scans
}

fn version_checks(ctx: &Ctx) -> Vec<Finding> {
    let scans = scan_sdks(ctx);
    let total_versions: usize = scans.iter().map(|scan| scan.versions).sum();
    let active = scans
        .iter()
        .filter(|scan| scan.active.is_some() && !scan.broken)
        .count();

    let mut findings = Vec::new();
    if scans.is_empty() {
        findings.push(Finding::new(
            "version.summary",
            Status::Ok,
            "No SDK versions installed yet.",
        ));
    } else {
        findings.push(
            Finding::new(
                "version.summary",
                Status::Ok,
                format!(
                    "{} {}, {} installed {}, {} active.",
                    scans.len(),
                    if scans.len() == 1 { "SDK" } else { "SDKs" },
                    total_versions,
                    if total_versions == 1 {
                        "version"
                    } else {
                        "versions"
                    },
                    active
                ),
            )
            .detail(ctx.versions_dir.display().to_string()),
        );
    }

    let mut broken = Vec::new();
    let mut outside = Vec::new();
    let mut link_errors = Vec::new();
    let mut empty = Vec::new();
    for scan in &scans {
        if let Some(error) = &scan.link_error {
            link_errors.push(format!("{}: {error}", scan.link.display()));
        }
        if scan.broken {
            if let Some(target) = &scan.active {
                broken.push(format!(
                    "{} -> {} (target missing)",
                    scan.link.display(),
                    target.display()
                ));
            }
            continue;
        }
        if let Some(target) = &scan.active {
            let target = fs::canonicalize(target).unwrap_or_else(|_| target.clone());
            let root = fs::canonicalize(&scan.root).unwrap_or_else(|_| scan.root.clone());
            if !path_under(&target, &root) {
                outside.push(format!("{} -> {}", scan.link.display(), target.display()));
            }
        }
        if scan.versions == 0 && scan.active.is_none() {
            empty.push(scan.root.display().to_string());
        }
    }

    if !broken.is_empty() {
        findings.push(
            Finding::new(
                "version.symlink",
                Status::Fail,
                "The active version link is broken.",
            )
            .details(cap(broken, 8))
            .fix("run `vmr use <sdk>@<version>` for an installed version, or uninstall the SDK"),
        );
    }
    if !outside.is_empty() {
        findings.push(
            Finding::new(
                "version.link_outside",
                Status::Warn,
                "An active version link points outside the versions directory.",
            )
            .details(cap(outside, 8))
            .fix("switch to an installed version with `vmr use`"),
        );
    }
    if !link_errors.is_empty() {
        findings.push(
            Finding::new(
                "version.link_error",
                Status::Warn,
                "An active version link could not be read.",
            )
            .details(cap(link_errors, 8))
            .fix("switch to an installed version with `vmr use`"),
        );
    }
    if !empty.is_empty() {
        findings.push(
            Finding::new(
                "version.empty",
                Status::Warn,
                "Version roots exist without any installed version.",
            )
            .details(cap(empty, 8))
            .fix("remove these directories if they are not needed"),
        );
    }
    findings.push(check_plugins(ctx));
    findings
}

fn check_plugins(ctx: &Ctx) -> Finding {
    let Ok(read_dir) = fs::read_dir(&ctx.plugins_dir) else {
        return Finding::new(
            "version.plugins",
            Status::Ok,
            "No plugins cached; they are downloaded on first use.",
        );
    };
    let mut plugins = 0usize;
    let mut has_common = false;
    for entry in read_dir.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".lua") {
            plugins += 1;
            if name == "_vmr_common.lua" {
                has_common = true;
            }
        }
    }
    if plugins == 0 {
        Finding::new(
            "version.plugins",
            Status::Ok,
            "No plugins cached; they are downloaded on first use.",
        )
    } else if !has_common {
        Finding::new(
            "version.plugins",
            Status::Warn,
            "The plugin directory is missing _vmr_common.lua.",
        )
        .detail(ctx.plugins_dir.display().to_string())
        .fix("run `vmr plugins -u` to refresh the plugin bundle")
    } else {
        Finding::new(
            "version.plugins",
            Status::Ok,
            format!("{plugins} plugin file(s) cached."),
        )
        .detail(ctx.plugins_dir.display().to_string())
    }
}

// ---------------------------------------------------------------------------
// Session checks
// ---------------------------------------------------------------------------

fn session_checks(ctx: &Ctx) -> Vec<Finding> {
    let nested_env = ctx.env.get(envs::ALLOW_NESTED_SESSIONS);
    let nested = ctx.conf.allow_nested_sessions || nested_env.and_then(truthy).unwrap_or(false);
    let mut finding = if ctx.session {
        Finding::new("session.mode", Status::Ok, "Session mode is active.")
    } else {
        Finding::new("session.mode", Status::Ok, "Global mode.")
    }
    .detail(format!(
        "nested sessions: {}",
        if nested { "allowed" } else { "not allowed" }
    ));
    if let Some(value) = nested_env
        && truthy(value).is_none()
    {
        finding = finding
            .with_status(Status::Warn)
            .with_message("VMR_ALLOW_NESTED_SESSIONS has an invalid value; it is treated as false.")
            .detail(format!("value: {value:?}"))
            .fix("unset it or use one of: true, false, 1, 0, yes, no, on, off");
    }
    vec![finding]
}

// ---------------------------------------------------------------------------
// Environment-file parsing
// ---------------------------------------------------------------------------

enum EnvLine {
    Path(String),
    Var(String),
    Unknown,
}

fn parse_env_line(kind: Kind, line: &str) -> EnvLine {
    match kind {
        Kind::Fish => {
            if let Some(rest) = line.strip_prefix("fish_add_path --global ") {
                return EnvLine::Path(unquote(rest));
            }
            if let Some(rest) = line.strip_prefix("fish_add_path ") {
                return EnvLine::Path(unquote(rest));
            }
            if let Some(rest) = line.strip_prefix("set --global ") {
                let mut parts = rest.splitn(2, ' ');
                let key = parts.next().unwrap_or_default();
                let value = parts.next().unwrap_or_default();
                return EnvLine::Var(format!("{key}={}", unquote(value)));
            }
            EnvLine::Unknown
        }
        _ => {
            let Some(rest) = line.strip_prefix("export ") else {
                return EnvLine::Unknown;
            };
            let Some((key, value)) = rest.split_once('=') else {
                return EnvLine::Unknown;
            };
            if key == "PATH" {
                let mut value = value.trim();
                for suffix in [":\"${PATH}\"", ":\"$PATH\"", ":${PATH}", ":$PATH"] {
                    if let Some(stripped) = value.strip_suffix(suffix) {
                        value = stripped;
                        break;
                    }
                }
                EnvLine::Path(unquote(value))
            } else {
                EnvLine::Var(format!("{key}={}", unquote(value)))
            }
        }
    }
}

fn env_file_vars(ctx: &Ctx) -> Vec<String> {
    let Ok(data) = fs::read_to_string(&ctx.env_file) else {
        return Vec::new();
    };
    data.lines()
        .filter_map(|raw| {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            match parse_env_line(ctx.kind, line) {
                EnvLine::Var(key) => key.split('=').next().map(str::to_string),
                _ => None,
            }
        })
        .collect()
}

fn foreign_syntax(kind: Kind, line: &str) -> bool {
    match kind {
        Kind::Fish => line.starts_with("export ") || line.contains("export PATH="),
        _ => line.starts_with("fish_add_path ") || line.starts_with("set --global "),
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let quoted = (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'');
        if quoted {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn expand_home(value: &str, home: &str) -> String {
    if let Some(rest) = value.strip_prefix("$HOME") {
        return format!("{home}{rest}");
    }
    if let Some(rest) = value.strip_prefix("${HOME}") {
        return format!("{home}{rest}");
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    value.to_string()
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn has_active_sdk(ctx: &Ctx) -> bool {
    scan_sdks(ctx)
        .iter()
        .any(|scan| scan.active.is_some() && !scan.broken)
}

fn managed_entry(ctx: &Ctx, entry: &str) -> bool {
    let path = Path::new(entry);
    path_under(path, &ctx.work_dir)
        || path_under(path, &ctx.versions_dir)
        || looks_vmr_managed(entry)
}

/// Recognizes the `<sdk>_versions/<sdk>` layout even after the install root moved, so stale
/// entries from a previous `VMR_SDK_INSTALLATION_DIR` are still reported.
fn looks_vmr_managed(value: &str) -> bool {
    let parts: Vec<&str> = value.split(['/', '\\']).collect();
    parts
        .windows(2)
        .any(|pair| pair[0].ends_with("_versions") && !pair[1].is_empty())
}

fn path_under(path: &Path, base: &Path) -> bool {
    #[cfg(windows)]
    {
        let path = normalize_path_text(&path.to_string_lossy());
        let base = normalize_path_text(&base.to_string_lossy());
        path == base || path.starts_with(&format!("{base}\\"))
    }
    #[cfg(not(windows))]
    {
        path == base || path.starts_with(base)
    }
}

fn normalize_path_text(value: &str) -> String {
    let trimmed = value.trim_end_matches(['/', '\\']);
    let trimmed = if trimmed.is_empty() { value } else { trimmed };
    #[cfg(windows)]
    {
        trimmed.replace('/', "\\").to_lowercase()
    }
    #[cfg(not(windows))]
    {
        trimmed.to_string()
    }
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => normalize_path_text(&a.to_string_lossy()) == normalize_path_text(&b.to_string_lossy()),
    }
}

fn under_canonical(path: &Path, base: &Path) -> bool {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let base = fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    path_under(&path, &base)
}

#[cfg(unix)]
fn same_file(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy()),
        _ => false,
    }
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
    let executable = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "exe" | "cmd" | "bat" | "com"
            )
        })
        .unwrap_or(false);
    executable
        && fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
}

#[cfg(not(windows))]
fn resolve_command(name: &str, entries: &[String]) -> Option<PathBuf> {
    for entry in entries {
        if entry.is_empty() {
            continue;
        }
        let path = Path::new(entry).join(name);
        if is_executable_file(&path) {
            return Some(path);
        }
    }
    None
}

#[cfg(windows)]
fn resolve_command(name: &str, entries: &[String]) -> Option<PathBuf> {
    let mut names = vec![name.to_string()];
    if Path::new(name).extension().is_none() {
        for extension in ["exe", "cmd", "bat", "com"] {
            names.push(format!("{name}.{extension}"));
        }
    }
    for entry in entries {
        if entry.is_empty() {
            continue;
        }
        for name in &names {
            let path = Path::new(entry).join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(unix)]
fn writable(path: &Path) -> Option<bool> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_os_str().as_bytes()).ok()?;
    Some(unsafe { libc::access(path.as_ptr(), libc::W_OK) } == 0)
}

#[cfg(not(unix))]
fn writable(_: &Path) -> Option<bool> {
    None
}

fn nearest_existing(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if candidate.exists() {
            return Some(candidate.to_path_buf());
        }
        current = candidate.parent();
    }
    None
}

fn truthy(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "t" | "yes" | "on" => Some(true),
        "0" | "false" | "f" | "no" | "off" | "" => Some(false),
        _ => None,
    }
}

fn has_url_scheme(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("http://") || value.starts_with("https://")
}

fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn redact(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "***".to_string();
    }
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars.iter().skip(chars.len() - 4).collect();
    format!("{head}…{tail}")
}

fn cap(mut items: Vec<String>, limit: usize) -> Vec<String> {
    if items.len() > limit {
        let extra = items.len() - limit;
        items.truncate(limit);
        items.push(format!("... and {extra} more"));
    }
    items
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

fn report_json(ctx: &Ctx, version: &str, sections: &[Section], summary: Summary) -> Value {
    json!({
        "schema": 1,
        "tool": "vmr",
        "version": version,
        "platform": ctx.platform,
        "sections": sections
            .iter()
            .map(|section| json!({
                "id": section.id,
                "title": section.title,
                "findings": section
                    .findings
                    .iter()
                    .map(|finding| json!({
                        "id": finding.id,
                        "status": finding.status.json(),
                        "message": finding.message,
                        "details": finding.details,
                        "fix": finding.fix,
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
        "summary": {
            "ok": summary.ok,
            "warn": summary.warn,
            "fail": summary.fail,
            "skip": summary.skip,
        },
    })
}

fn print_text(
    ctx: &Ctx,
    version: &str,
    sections: &[Section],
    summary: Summary,
    emoji: bool,
    quiet: bool,
) {
    if emoji {
        println!("🔍 vmr doctor {version} · {}", ctx.platform);
    } else {
        println!("vmr doctor {version} ({})", ctx.platform);
    }
    println!();

    for section in sections {
        let visible: Vec<&Finding> = section
            .findings
            .iter()
            .filter(|finding| !quiet || matches!(finding.status, Status::Warn | Status::Fail))
            .collect();
        if visible.is_empty() {
            continue;
        }
        if emoji {
            println!("{} {}", section.emoji, section.title);
        } else {
            println!("{}", section.title);
        }
        for finding in visible {
            let label = if emoji {
                finding.status.emoji().to_string()
            } else {
                format!("[{}]", finding.status.label())
            };
            println!("  {label} {}", finding.message);
            for detail in &finding.details {
                println!("     {detail}");
            }
            if let Some(fix) = &finding.fix {
                if emoji {
                    println!("     🔧 {fix}");
                } else {
                    println!("     [FIX] {fix}");
                }
            }
        }
        println!();
    }

    if emoji {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let mut parts = vec![
            format!("❌ {}", count_label(summary.fail, "error", "errors")),
            format!("⚠️ {}", count_label(summary.warn, "warning", "warnings")),
            format!("✅ {}", count_label(summary.ok, "ok", "ok")),
        ];
        if summary.skip > 0 {
            parts.push(format!("⏭️ {} skipped", summary.skip));
        }
        println!("{}", parts.join("   "));
    } else {
        println!("{}", "-".repeat(28));
        let mut parts = vec![
            format!("FAIL: {}", summary.fail),
            format!("WARN: {}", summary.warn),
            format!("OK: {}", summary.ok),
        ];
        if summary.skip > 0 {
            parts.push(format!("SKIP: {}", summary.skip));
        }
        println!("{}", parts.join("   "));
    }
}

fn count_label(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("vmr-doctor-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn ctx(root: &Path, kind: Kind, env: Env, conf: VMRConf) -> Ctx {
        Ctx::for_test(root, kind, env, conf)
    }

    fn find<'a>(findings: &'a [Finding], id: &str) -> &'a Finding {
        findings
            .iter()
            .find(|finding| finding.id == id)
            .unwrap_or_else(|| panic!("missing finding {id}"))
    }

    #[cfg(unix)]
    fn make_exec(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    fn add_active_sdk(versions: &Path, sdk: &str, version: &str, command: &str) -> PathBuf {
        let bin = versions.join(format!("{sdk}_versions/{version}/bin"));
        make_exec(&bin.join(command));
        std::os::unix::fs::symlink(version, versions.join(format!("{sdk}_versions/{sdk}")))
            .unwrap();
        bin
    }

    #[cfg(unix)]
    #[test]
    fn shared_commands_between_active_sdks_are_not_errors() {
        let root = temp_root("shared-commands");
        let versions = root.join(".vmr/versions");
        let a = add_active_sdk(&versions, "a", "a-1", "tool");
        let b = add_active_sdk(&versions, "b", "b-1", "tool");
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let entries = vec![
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
        ];
        let findings = check_active_commands(&ctx, &entries);
        assert!(
            findings.iter().all(|finding| finding.id != "path.shadow"),
            "another active SDK must not be reported as external shadowing"
        );
        let shared = find(&findings, "path.shared");
        assert_eq!(shared.status, Status::Ok);
        assert_eq!(
            shared.message,
            "1 command is provided by more than one active SDK; PATH order decides."
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn external_toolchain_shadow_is_still_an_error() {
        let root = temp_root("external-shadow");
        let versions = root.join(".vmr/versions");
        let active = add_active_sdk(&versions, "go", "go-1.22.0", "go");
        let external = root.join("system");
        make_exec(&external.join("go"));
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let entries = vec![
            external.to_string_lossy().into_owned(),
            active.to_string_lossy().into_owned(),
        ];
        let findings = check_active_commands(&ctx, &entries);
        let shadow = find(&findings, "path.shadow");
        assert_eq!(shadow.status, Status::Fail);
        assert!(
            shadow
                .details
                .iter()
                .any(|detail| detail.contains("system"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_conf_override() {
        let root = temp_root("conf-override");
        let env = Env::from_pairs([(envs::REVERSE_PROXY, "https://mirror.example/proxy/")]);
        let conf = VMRConf {
            reverse_proxy: "https://proxy.example/proxy/".to_string(),
            ..VMRConf::default()
        };
        let ctx = ctx(&root, Kind::Zsh, env, conf);
        let findings = env_checks(&ctx);
        let finding = find(&findings, "env.conf_override");
        assert_eq!(finding.status, Status::Warn);
        assert!(
            finding
                .details
                .iter()
                .any(|detail| detail.contains("shell="))
        );
        assert!(
            finding
                .details
                .iter()
                .any(|detail| detail.contains("conf="))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn local_proxy_disables_reverse_proxy() {
        let root = temp_root("proxy");
        let env = Env::from_pairs([
            (envs::LOCAL_PROXY, "http://127.0.0.1:7890"),
            (envs::REVERSE_PROXY, "https://proxy.example/"),
        ]);
        let ctx = ctx(&root, Kind::Zsh, env, VMRConf::default());
        let finding = check_proxy(&ctx);
        assert_eq!(finding.status, Status::Warn);
        assert!(finding.message.contains("reverse proxy is ignored"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_missing_shell_init_line() {
        let root = temp_root("init-missing");
        fs::write(root.join(".zshrc"), "# empty\n").unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_init_line(&ctx);
        assert_eq!(
            find(findings.as_slice(), "shell.init_line").status,
            Status::Fail
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn accepts_present_shell_init_line() {
        let root = temp_root("init-present");
        let line = Shell::new(Kind::Zsh).init_line();
        fs::write(root.join(".zshrc"), format!("{line}\n")).unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_init_line(&ctx);
        assert_eq!(
            find(findings.as_slice(), "shell.init_line").status,
            Status::Ok
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(windows))]
    #[test]
    fn detects_stale_env_path() {
        let root = temp_root("stale-path");
        let work = root.join(".vmr");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("vmr.sh"),
            "export PATH=\"$HOME/.vmr/versions/go_versions/go/1.22.0/bin\":\"${PATH}\"\n",
        )
        .unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_env_file(&ctx);
        let finding = find(findings.as_slice(), "shell.env_dead");
        assert_eq!(finding.status, Status::Fail);
        assert!(
            finding
                .details
                .iter()
                .any(|detail| detail.contains("go/1.22.0"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(windows))]
    #[test]
    fn stale_paths_from_a_previous_install_root_are_managed() {
        let root = temp_root("old-root");
        let work = root.join(".vmr");
        fs::create_dir_all(&work).unwrap();
        let old = "/old/vmr/versions/go_versions/go/bin";
        fs::write(
            work.join("vmr.sh"),
            format!("export PATH=\"{old}\":\"${{PATH}}\"\n"),
        )
        .unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_env_file(&ctx);
        assert_eq!(
            find(findings.as_slice(), "shell.env_dead").status,
            Status::Fail
        );
        let entries = vec![old.to_string()];
        assert_eq!(check_path_missing(&ctx, &entries).status, Status::Fail);
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(windows))]
    #[test]
    fn combined_path_value_is_split_into_entries() {
        let root = temp_root("combined-path");
        let work = root.join(".vmr");
        fs::create_dir_all(work.join("versions/mini_versions/mini/bin")).unwrap();
        fs::create_dir_all(work.join("versions/mini_versions/mini/condabin")).unwrap();
        fs::write(
            work.join("vmr.sh"),
            "export PATH=\"$HOME/.vmr/versions/mini_versions/mini/bin:$HOME/.vmr/versions/mini_versions/mini/condabin\":\"${PATH}\"\n",
        )
        .unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_env_file(&ctx);
        assert!(
            findings
                .iter()
                .all(|finding| finding.id != "shell.env_dead"),
            "a colon-separated PATH value must be split before existence checks"
        );
        let summary = find(&findings, "shell.env_file");
        assert_eq!(summary.status, Status::Ok);
        assert!(
            summary.message.contains("2 PATH entries"),
            "{}",
            summary.message
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(windows))]
    #[test]
    fn unloaded_integration_suggests_reloading_the_shell() {
        let root = temp_root("cd-reload");
        let line = Shell::new(Kind::Zsh).init_line();
        fs::write(root.join(".zshrc"), format!("{line}\n")).unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let finding = check_cd_hook(&ctx);
        assert_eq!(finding.status, Status::Warn);
        assert_eq!(finding.fix.as_deref(), Some("exec zsh"));
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(not(windows))]
    #[test]
    fn detects_foreign_env_file_syntax() {
        let root = temp_root("foreign");
        let work = root.join(".vmr");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("vmr.sh"),
            "fish_add_path --global \"$HOME/.cargo/bin\"\n",
        )
        .unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = check_env_file(&ctx);
        assert_eq!(
            find(findings.as_slice(), "shell.env_syntax").status,
            Status::Fail
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn detects_broken_active_link() {
        let root = temp_root("broken-link");
        let versions = root.join(".vmr/versions");
        let sdk = versions.join("go_versions");
        fs::create_dir_all(&sdk).unwrap();
        std::os::unix::fs::symlink("go-1.22.0", sdk.join("go")).unwrap();
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let findings = version_checks(&ctx);
        assert_eq!(
            find(findings.as_slice(), "version.symlink").status,
            Status::Fail
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn redacts_tokens() {
        assert_eq!(redact("short"), "***");
        let token = "ghp_1234567890abcdef";
        let redacted = redact(token);
        assert!(!redacted.contains(token));
        assert!(redacted.starts_with("ghp_"));
        assert!(redacted.ends_with("cdef"));
    }

    #[test]
    fn parses_env_lines_for_each_shell() {
        match parse_env_line(
            Kind::Zsh,
            "export PATH=\"$HOME/.vmr/versions/go_versions/go/1.22.0/bin\":\"${PATH}\"",
        ) {
            EnvLine::Path(path) => {
                assert_eq!(path, "$HOME/.vmr/versions/go_versions/go/1.22.0/bin")
            }
            _ => panic!("expected a PATH line"),
        }
        match parse_env_line(Kind::Fish, "fish_add_path --global \"$HOME/.cargo/bin\"") {
            EnvLine::Path(path) => assert_eq!(path, "$HOME/.cargo/bin"),
            _ => panic!("expected a PATH line"),
        }
        match parse_env_line(
            Kind::Zsh,
            "export GOROOT=\"$HOME/.vmr/versions/go_versions/go\"",
        ) {
            EnvLine::Var(key) => assert!(key.starts_with("GOROOT=")),
            _ => panic!("expected a variable line"),
        }
    }

    #[test]
    fn json_report_has_stable_status_strings() {
        let root = temp_root("json");
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let sections = vec![Section {
            id: "test",
            title: "Test",
            emoji: "🧪",
            findings: vec![
                Finding::new("a", Status::Ok, "ok"),
                Finding::new("b", Status::Warn, "warn"),
                Finding::new("c", Status::Fail, "fail"),
                Finding::new("d", Status::Skip, "skip"),
            ],
        }];
        let value = report_json(&ctx, "0.0.0", &sections, Summary::of(&sections));
        let statuses: Vec<&str> = value["sections"][0]["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| finding["status"].as_str().unwrap())
            .collect();
        assert_eq!(statuses, ["ok", "warn", "fail", "skip"]);
        assert_eq!(value["summary"]["fail"], 1);
        let text = serde_json::to_string(&value).unwrap();
        assert!(!text.contains('✅'));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn path_duplicates_only_cover_vmr_entries() {
        let root = temp_root("duplicates");
        let ctx = ctx(&root, Kind::Zsh, Env::default(), VMRConf::default());
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/bin".to_string(),
            ctx.work_dir.to_string_lossy().into_owned(),
            ctx.work_dir.to_string_lossy().into_owned(),
        ];
        let finding = check_path_duplicates(&ctx, &entries);
        assert_eq!(finding.status, Status::Warn);
        assert_eq!(finding.details.len(), 1);
        assert!(finding.details[0].contains(".vmr"));
        let _ = fs::remove_dir_all(root);
    }
}
