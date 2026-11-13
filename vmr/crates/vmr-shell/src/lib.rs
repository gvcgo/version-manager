//! vmr-shell: shell hook environment injection and automatic cd-hook switching (plan.md §3.8, requirement 7).
//!
//! Mirrors Go `internal/shell/`: a shared scheme for bash/zsh/fish —
//! - Shell environment file `~/.vmr/vmr.sh` (bash/zsh) or `~/.vmr/vmr.fish` (fish);
//! - `update_vmr_shell_file`: idempotent replacement between `# cd hook start … # cd hook end` markers;
//! - rc files (`.bashrc`/`.zshrc`/`config.fish`) get a source block appended (`VM_DISABLE` guard);
//! - `PackPath`/`PackEnv` line injection / prefix-based removal (Set/Unset).
//!
//! Session contract: skip injection when `VM_DISABLE=111`; the cd hook calls `vmr cdhook` for automatic switching.
//! A `.vmr.lock` child shell additionally gets a session hook that exits when `cd` leaves the
//! project and records the destination for the outer hook (see [`paths::session_exit_file`]).
//! A child process cannot change its parent's environment, so the integration defined by
//! `vmr init` includes a `vmr` wrapper: after a successful `vmr use` / `vmr uninstall` it
//! reloads the env file into the current shell. On bash/zsh it first drops vmr-managed PATH
//! entries and unsets variables listed in `~/.vmr/env_keys.txt`, so repeated refreshes are
//! idempotent and an uninstalled variable does not linger. On Windows the same wrapper is
//! generated for PowerShell and re-reads `HKCU\Environment` (`vmrsource`).

use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::paths;

mod scripts;
pub use scripts::ZSH_COMPLETION_TAIL;
#[cfg(windows)]
mod win_patch;
#[cfg(windows)]
mod win_reg;

pub const VM_DISABLE_ENV_NAME: &str = "VM_DISABLE";
pub const VM_CD_INIT_ENV_NAME: &str = "VMR_CD_INIT";
/// Project directory cached by the cd hook; `vmr cdhook` writes it to
/// [`paths::cd_cache_file`] and the hook reads it back into this variable.
pub const VM_CD_CACHE_ENV_NAME: &str = "VMR_CD_CACHE";
pub const MODE_PERM: u32 = 0o644;

/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bash,
    Zsh,
    Fish,
    /// Windows PowerShell (not a primary unix path).
    PowerShell,
}

impl Kind {
    fn env_file_name(self) -> &'static str {
        match self {
            Kind::Fish => "vmr.fish",
            _ => "vmr.sh",
        }
    }

    /// Canonical name accepted by `vmr init <shell>`.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Bash => "bash",
            Kind::Zsh => "zsh",
            Kind::Fish => "fish",
            Kind::PowerShell => "powershell",
        }
    }

    /// Parses a shell name; `pwsh`/`ps1` are accepted for PowerShell.
    pub fn parse(s: &str) -> Option<Kind> {
        match s.trim().to_lowercase().as_str() {
            "bash" => Some(Kind::Bash),
            "zsh" => Some(Kind::Zsh),
            "fish" => Some(Kind::Fish),
            "powershell" | "pwsh" | "ps1" => Some(Kind::PowerShell),
            _ => None,
        }
    }

    /// Static cd hook body, emitted by `vmr init` and evaluated from the rc file
    /// (bash/zsh and fish; PowerShell has no shell hook).
    fn hook_script(self, install_dir: &str, home_dir: &str) -> String {
        let inst = script_path(self, install_dir, home_dir);
        let cache_file = paths::cd_cache_file();
        let cache = script_path(self, &cache_file.to_string_lossy(), home_dir);
        let exit = self.session_exit_path(home_dir);
        match self {
            Kind::Fish => fill(
                scripts::HOOK_FISH,
                &[
                    ("__VMR_INSTALL__", &inst),
                    ("__VMR_CD_CACHE__", &cache),
                    ("__VMR_SESSION_EXIT__", &exit),
                ],
            ),
            Kind::PowerShell => fill(
                scripts::HOOK_POWERSHELL,
                &[
                    ("__VMR_CD_CACHE__", &ps_quote(&cache_file.to_string_lossy())),
                    (
                        "__VMR_SESSION_EXIT__",
                        &ps_quote(&paths::session_exit_file().to_string_lossy()),
                    ),
                ],
            ),
            _ => {
                // Unix gets the install directory from `_vmr_refresh`; git-bash has no env
                // file refresh, so it keeps the inline PATH line.
                #[cfg(windows)]
                {
                    format!(
                        "export PATH=\"{inst}\":\"${{PATH}}\"\n\n{}",
                        fill(
                            scripts::HOOK_SH,
                            &[
                                ("__VMR_CD_CACHE__", &cache),
                                ("__VMR_SESSION_EXIT__", &exit),
                            ]
                        )
                    )
                }
                #[cfg(not(windows))]
                {
                    fill(
                        scripts::HOOK_SH,
                        &[
                            ("__VMR_CD_CACHE__", &cache),
                            ("__VMR_SESSION_EXIT__", &exit),
                        ],
                    )
                }
            }
        }
    }

    /// Path of the session exit file as written in the generated script.
    fn session_exit_path(self, home_dir: &str) -> String {
        script_path(
            self,
            &paths::session_exit_file().to_string_lossy(),
            home_dir,
        )
    }

    /// `.vmr.lock` session hook: leaves the child shell when `cd` goes outside the project
    /// recorded in `VMR_SESSION_PROJECT` and writes the destination for the outer hook.
    fn session_hook_script(self, home_dir: &str) -> String {
        let exit = match self {
            Kind::PowerShell => ps_quote(&paths::session_exit_file().to_string_lossy()),
            _ => self.session_exit_path(home_dir),
        };
        let template = match self {
            Kind::Fish => scripts::SESSION_HOOK_FISH,
            Kind::PowerShell => scripts::SESSION_HOOK_POWERSHELL,
            _ => scripts::SESSION_HOOK_SH,
        };
        fill(template, &[("__VMR_SESSION_EXIT__", &exit)])
    }
}

/// Shell path display: a path under home becomes `$HOME/...` (quoted by the callers where
/// needed). Mirrors Go `FormatPathString` but with `$HOME` instead of `~`, so generated
/// lines behave identically when quoted.
pub fn format_path_string(p: &str, home_dir: &str) -> String {
    if let Some(rest) = p.strip_prefix(home_dir) {
        return format!("$HOME{rest}");
    }
    p.to_string()
}

/// Path form used in generated bash/zsh/fish scripts. Git-bash cannot use `C:\...` in
/// `PATH` (the drive colon is a separator), so Windows paths are converted to mingw form.
fn script_path(kind: Kind, p: &str, home_dir: &str) -> String {
    #[cfg(windows)]
    {
        if matches!(kind, Kind::Bash | Kind::Zsh | Kind::Fish) {
            let p = convert_to_mingw(p);
            let home = convert_to_mingw(home_dir);
            if !home.is_empty() {
                if let Some(rest) = p.strip_prefix(&home) {
                    if rest.is_empty() || rest.starts_with('/') {
                        return format!("$HOME{rest}");
                    }
                }
            }
            return p;
        }
    }
    let _ = kind;
    format_path_string(p, home_dir)
}

/// Converts `C:\a\b` to `/c/a/b` (Go `ConvertWindowsPathToMingwPath`; used by the Windows
/// git-bash layer).
pub fn convert_to_mingw(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let normalized = path.replace('\\', "/").replace(':', "");
    if normalized.starts_with('/') {
        return normalized;
    }
    let mut parts: Vec<String> = normalized.split('/').map(str::to_string).collect();
    if let Some(first) = parts.first_mut() {
        *first = first.to_lowercase();
    }
    format!("/{}", parts.join("/"))
}

/// Switches the Windows user `PATH` to the shim directory: removes every legacy
/// versions-directory entry (absolute or `%VMR_VERSIONS%\...`) plus its git-bash mirror,
/// then prepends `dir`. Idempotent; called on every global `vmr use`.
#[cfg(windows)]
pub fn use_shim_path(dir: &str) {
    win_reg::remove_version_entries();
    win_patch::clear_version_exports();
    win_reg::set_path(dir);
}

/// Legacy `~` form of a home path, used to clean up lines written by older versions.
#[cfg(not(windows))]
fn tilde_path(p: &str, home_dir: &str) -> String {
    if let Some(rest) = p.strip_prefix(home_dir) {
        return format!("~{rest}");
    }
    p.to_string()
}

/// Detect the current shell (SHELL env; docker without SHELL → bash).
/// Windows has no reliable `SHELL` in PowerShell/cmd, so those default to PowerShell
/// (git-bash still reports bash via `SHELL`).
pub fn detect_kind() -> Kind {
    let shell = std::env::var("SHELL").unwrap_or_default().to_lowercase();
    if shell.contains("zsh") {
        return Kind::Zsh;
    }
    if shell.contains("fish") {
        return Kind::Fish;
    }
    if shell.contains("bash") || shell.ends_with("sh") || shell.ends_with("sh.exe") {
        return Kind::Bash;
    }
    #[cfg(windows)]
    {
        Kind::PowerShell
    }
    #[cfg(not(windows))]
    {
        Kind::Bash
    }
}

/// Substitutes `__VMR_*__` placeholders in an embedded script template.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |s, (key, value)| {
        s.replace(key, value)
    })
}

/// Indents every non-empty line of `block` with `pad`.
fn indent_block(block: &str, pad: &str) -> String {
    block
        .lines()
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("{pad}{l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Removes a `start … end` marked block (inclusive) from the rc text.
fn strip_marked(data: &str, start: &str, end: &str) -> String {
    let mut out = String::with_capacity(data.len());
    let mut in_block = false;
    for line in data.lines() {
        if in_block {
            if line.contains(end) {
                in_block = false;
            }
            continue;
        }
        if line.contains(start) {
            in_block = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Legacy rc blocks (removed when migrating to the `vmr init` line).
const LEGACY_BLOCKS: [(&str, &str); 6] = [
    ("# vm_envs start", "# vm_envs end"),
    ("# vmr source alias start", "# vmr source alias end"),
    ("# VMR Completions (Bash)", "# VMR Completions (Bash)"),
    ("# VMR Completions (Zsh)", "# VMR Completions (Zsh)"),
    ("# VMR Completions (Fish)", "# VMR Completions (Fish)"),
    (
        "# VMR Completions (PowerShell)",
        "# VMR Completions (PowerShell)",
    ),
];

/// Strips the legacy source blocks from the rc text.
fn strip_legacy_blocks(data: &str) -> String {
    LEGACY_BLOCKS
        .iter()
        .fold(data.to_string(), |d, (start, end)| {
            strip_marked(&d, start, end)
        })
}

/// Ensures the rc file contains exactly one `eval "$(vmr init <shell>)"` line, migrating the
/// legacy `# vm_envs` / `# vmr source alias` blocks away (idempotent).
fn ensure_init_line(conf_path: &Path, line: &str) {
    let data = fs::read_to_string(conf_path).unwrap_or_default();
    let stripped = strip_legacy_blocks(&data);
    let has_line = stripped.lines().any(|l| l.trim() == line.trim());
    let mut out = stripped.trim_end().to_string();
    if !has_line {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
    }
    out.push('\n');
    if out != data {
        if let Some(parent) = conf_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(conf_path, out);
    }
}

/// The script printed by `vmr init <shell>` (zoxide-style integration).
pub fn init_script(kind: Kind) -> String {
    Shell::new(kind).init_script()
}

/// Reads a value from the Windows user environment (`HKCU\Environment`).
#[cfg(windows)]
pub fn user_env(key: &str) -> Option<String> {
    win_reg::get_env(key)
}

/// Quotes a value for a PowerShell single-quoted literal.
fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// List of non-PATH variables vmr has written, read by `_vmr_refresh`. Keys stay listed
/// after `unset_env`, so a refresh clears variables an uninstall removed.
#[cfg(not(windows))]
fn env_keys_path() -> PathBuf {
    paths::work_dir().join("env_keys.txt")
}

/// Appends `key` to the env-key list unless it is already present (one key per line).
#[cfg(not(windows))]
fn merge_env_key(data: &str, key: &str) -> String {
    if data.lines().any(|l| l.trim() == key) {
        return data.to_string();
    }
    let mut out = data.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(key);
    out.push('\n');
    out
}

#[cfg(not(windows))]
fn record_env_key(key: &str) {
    if key.is_empty() || key.eq_ignore_ascii_case("PATH") {
        return;
    }
    let path = env_keys_path();
    let data = fs::read_to_string(&path).unwrap_or_default();
    let out = merge_env_key(&data, key);
    if out != data {
        let _ = fs::write(path, out);
    }
}

/// Shell environment operator (non-Windows).
#[derive(Debug, Clone)]
pub struct Shell {
    pub kind: Kind,
    home: String,
}

impl Shell {
    pub fn new(kind: Kind) -> Self {
        let home = dirs_home();
        Shell { kind, home }
    }

    pub fn detect() -> Self {
        let mut sh = Shell::new(detect_kind());
        if cfg!(windows) {
            sh.kind = Kind::PowerShell;
        }
        sh
    }

    /// Path of the rc file.
    pub fn conf_path(&self) -> PathBuf {
        match self.kind {
            Kind::Fish => PathBuf::from(&self.home).join(".config/fish/config.fish"),
            Kind::Zsh => PathBuf::from(&self.home).join(".zshrc"),
            Kind::PowerShell => powershell_profile_path(&self.home),
            _ => PathBuf::from(&self.home).join(".bashrc"),
        }
    }

    /// Home directory used for rc files and `$HOME`-relative paths.
    pub fn home(&self) -> &str {
        &self.home
    }

    /// Path of the vmr shell environment file (`~/.vmr/vmr.sh|vmr.fish`).
    pub fn vm_env_conf_path(&self) -> PathBuf {
        paths::work_dir_path().join(self.kind.env_file_name())
    }

    #[cfg(not(windows))]
    fn env_file_text(&self) -> String {
        fs::read_to_string(self.vm_env_conf_path()).unwrap_or_default()
    }

    #[cfg(not(windows))]
    fn write_env_file(&self, data: &str) {
        let path = self.vm_env_conf_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, data.trim_start_matches('\n'));
    }

    pub fn pack_path(&self, path: &str) -> String {
        match self.kind {
            Kind::Fish => format!("fish_add_path --global \"{path}\""),
            _ => format!("export PATH=\"{path}\":\"${{PATH}}\""),
        }
    }

    /// Pre-`$HOME` form (unquoted `~`), used to remove lines written by older versions.
    #[cfg(not(windows))]
    fn pack_path_legacy(&self, path: &str) -> String {
        match self.kind {
            Kind::Fish => format!("fish_add_path --global {path}"),
            _ => format!("export PATH={path}:\"${{PATH}}\""),
        }
    }

    pub fn pack_env(&self, key: &str, value: &str) -> String {
        match self.kind {
            Kind::Fish => {
                if value.is_empty() {
                    format!("set --global {key} ")
                } else {
                    format!("set --global {key} \"{value}\"")
                }
            }
            _ => {
                if value.is_empty() {
                    format!("export {key}=")
                } else {
                    format!("export {key}=\"{value}\"")
                }
            }
        }
    }

    /// Prefix shared by every line that sets `key`, whatever the value quoting.
    #[cfg(not(windows))]
    fn env_prefix(&self, key: &str) -> String {
        match self.kind {
            Kind::Fish => format!("set --global {key} "),
            _ => format!("export {key}="),
        }
    }

    pub fn set_path(&self, path: &str) {
        #[cfg(windows)]
        {
            win_reg::set_path(path);
        }
        #[cfg(not(windows))]
        {
            let line = self.pack_path(&format_path_string(path, &self.home));
            let legacy = self.pack_path_legacy(&tilde_path(path, &self.home));
            let original = self.env_file_text();
            let mut data = original.replace(&legacy, "");
            if !data.contains(&line) {
                data = format!("{}\n{line}", data.trim_end());
            }
            if data != original {
                self.write_env_file(&data);
            }
        }
    }

    pub fn unset_path(&self, path: &str) {
        #[cfg(windows)]
        {
            win_reg::unset_path(path);
        }
        #[cfg(not(windows))]
        {
            let line = self.pack_path(&format_path_string(path, &self.home));
            let legacy = self.pack_path_legacy(&tilde_path(path, &self.home));
            let data = self.env_file_text();
            let out = data
                .replace(&line, "")
                .replace(&legacy, "")
                .replace("\n\n", "\n");
            self.write_env_file(&out);
        }
    }

    pub fn set_env(&self, key: &str, value: &str) {
        #[cfg(windows)]
        {
            win_reg::set_env(key, value);
        }
        #[cfg(not(windows))]
        {
            record_env_key(key);
            let data = self.env_file_text();
            let env_line = self.pack_env(key, &format_path_string(value, &self.home));
            if !data.contains(&env_line) {
                let out = format!("{}\n{env_line}", data.trim_end());
                self.write_env_file(&out);
            }
        }
    }

    pub fn unset_env(&self, key: &str) {
        #[cfg(windows)]
        {
            win_reg::unset_env(key);
        }
        #[cfg(not(windows))]
        {
            record_env_key(key);
            let prefix = self.env_prefix(key);
            let data = self.env_file_text();
            let mut out = String::new();
            for line in data.lines() {
                if line.trim_start().starts_with(prefix.trim_start()) {
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            self.write_env_file(&out.replace("\n\n", "\n"));
        }
    }

    /// The rc line that evaluates the integration (`eval "$(vmr init zsh)"` etc.).
    pub fn init_line(&self) -> String {
        match self.kind {
            Kind::Fish => "vmr init fish | source".to_string(),
            Kind::PowerShell => {
                "Invoke-Expression (& vmr init powershell | Out-String)".to_string()
            }
            _ => format!("eval \"$(vmr init {})\"", self.kind.name()),
        }
    }

    /// The integration script printed by `vmr init <shell>`: sources the env file, emits
    /// the static cd hook, then defines `svmr` (leave a session and reload the config).
    /// `VM_DISABLE` guards the whole hook so session shells stay untouched.
    ///
    /// On bash/zsh/fish the script also defines `_vmr_refresh` and a `vmr` wrapper, so a
    /// successful `vmr use` / `vmr uninstall` updates the current shell (idempotently).
    pub fn init_script(&self) -> String {
        let env_file = script_path(
            self.kind,
            &self.vm_env_conf_path().to_string_lossy(),
            &self.home,
        );
        let conf = script_path(self.kind, &self.conf_path().to_string_lossy(), &self.home);
        let install = paths::work_dir().to_string_lossy().into_owned();
        let hook = indent_block(&self.kind.hook_script(&install, &self.home), "    ");
        let session_hook = self.kind.session_hook_script(&self.home);
        #[cfg(not(windows))]
        let keys_file = script_path(
            self.kind,
            &paths::work_dir().join("env_keys.txt").to_string_lossy(),
            &self.home,
        );
        let ps_refresh = indent_block(
            &fill(
                scripts::REFRESH_POWERSHELL,
                &[
                    (
                        "__VMR_VERSIONS_DIR__",
                        &ps_quote(&paths::versions_dir().to_string_lossy()),
                    ),
                    (
                        "__VMR_ENV_KEYS__",
                        &ps_quote(&paths::work_dir().join("env_keys.txt").to_string_lossy()),
                    ),
                ],
            ),
            "    ",
        );
        match self.kind {
            Kind::Fish => {
                #[cfg(windows)]
                {
                    fill(
                        scripts::INIT_FISH_WINDOWS,
                        &[
                            ("__VMR_ENV_FILE__", &env_file),
                            ("__VMR_HOOK__", &hook),
                            ("__VMR_SESSION_HOOK__", &session_hook),
                            ("__VMR_CONF__", &conf),
                        ],
                    )
                }
                #[cfg(not(windows))]
                {
                    let refresh = indent_block(
                        &fill(
                            scripts::REFRESH_FISH,
                            &[
                                ("__VMR_ENV_FILE__", &env_file),
                                ("__VMR_ENV_KEYS__", &keys_file),
                            ],
                        ),
                        "    ",
                    );
                    let wrapper = indent_block(scripts::WRAPPER_FISH, "    ");
                    fill(
                        scripts::INIT_FISH,
                        &[
                            ("__VMR_REFRESH__", &refresh),
                            ("__VMR_WRAPPER__", &wrapper),
                            ("__VMR_HOOK__", &hook),
                            ("__VMR_SESSION_HOOK__", &session_hook),
                            ("__VMR_CONF__", &conf),
                        ],
                    )
                }
            }
            Kind::PowerShell => fill(
                scripts::INIT_POWERSHELL,
                &[
                    ("__VMR_SESSION_HOOK__", &session_hook),
                    ("__VMR_HOOK__", &hook),
                    ("__VMR_REFRESH__", &ps_refresh),
                ],
            ),
            _ => {
                #[cfg(windows)]
                {
                    fill(
                        scripts::INIT_SH_WINDOWS,
                        &[
                            ("__VMR_ENV_FILE__", &env_file),
                            ("__VMR_HOOK__", &hook),
                            ("__VMR_SESSION_HOOK__", &session_hook),
                            ("__VMR_CONF__", &conf),
                        ],
                    )
                }
                #[cfg(not(windows))]
                {
                    let refresh = indent_block(
                        &fill(
                            scripts::REFRESH_SH,
                            &[
                                ("__VMR_ENV_FILE__", &env_file),
                                ("__VMR_ENV_KEYS__", &keys_file),
                            ],
                        ),
                        "    ",
                    );
                    let wrapper = indent_block(scripts::WRAPPER_SH, "    ");
                    fill(
                        scripts::INIT_SH,
                        &[
                            ("__VMR_REFRESH__", &refresh),
                            ("__VMR_WRAPPER__", &wrapper),
                            ("__VMR_HOOK__", &hook),
                            ("__VMR_SESSION_HOOK__", &session_hook),
                            ("__VMR_CONF__", &conf),
                        ],
                    )
                }
            }
        }
    }

    /// Removes the legacy cd-hook section (and stray install-path lines) from the env file;
    /// the hook is now emitted inline by `vmr init`, so the file only keeps dynamic env lines.
    #[cfg(not(windows))]
    fn strip_hook_section(&self, env_path: &Path, install: &str) {
        let Ok(data) = fs::read_to_string(env_path) else {
            return;
        };
        let new_line = self.pack_path(&format_path_string(install, &self.home));
        let old_line = self.pack_path_legacy(&tilde_path(install, &self.home));
        let out = strip_marked(&data, "# cd hook start", "# cd hook end")
            .replace(&new_line, "")
            .replace(&old_line, "");
        let out = out.trim();
        let out = if out.is_empty() {
            String::new()
        } else {
            format!("{out}\n")
        };
        if out != data {
            let _ = fs::write(env_path, out);
        }
    }

    /// Strip the hook from the shell environment file and ensure the rc evaluates
    /// `vmr init` instead of the legacy source blocks. On Windows the environment lives in
    /// the registry, so only the profile line is maintained.
    pub fn write_vm_env_to_shell(&self) {
        #[cfg(windows)]
        {
            ensure_init_line(&self.conf_path(), &self.init_line());
        }
        #[cfg(not(windows))]
        {
            let install = paths::work_dir().to_string_lossy().into_owned();
            let env_file = self.vm_env_conf_path();
            self.strip_hook_section(&env_file, &install);
            if !env_file.exists() {
                if let Some(parent) = env_file.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&env_file, "");
            }
            ensure_init_line(&self.conf_path(), &self.init_line());
        }
    }
}

fn dirs_home() -> String {
    #[cfg(windows)]
    {
        // git-bash sets HOME to a POSIX path (/c/Users/...), which is not a valid Windows
        // profile path; always prefer the native profile for registry/profile write-back.
        std::env::var("USERPROFILE")
            .ok()
            .or_else(|| std::env::var("HOME").ok())
            .unwrap_or_else(|| ".".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .ok()
            .unwrap_or_else(|| ".".to_string())
    }
}

/// Windows PowerShell profile path (mirrors Go `win.go`).
fn powershell_profile_path(home: &str) -> PathBuf {
    PathBuf::from(home).join("Documents/WindowsPowerShell/profile.ps1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn format_path_replaces_home() {
        assert_eq!(format_path_string("/home/u/.vmr", "/home/u"), "$HOME/.vmr");
        assert_eq!(format_path_string("/opt/x", "/home/u"), "/opt/x");
    }

    #[test]
    fn converts_windows_paths_to_mingw() {
        assert_eq!(
            convert_to_mingw(r"C:\Users\me\.vmr\versions"),
            "/c/Users/me/.vmr/versions"
        );
        assert_eq!(convert_to_mingw(""), "");
        assert_eq!(convert_to_mingw(r"D:\x"), "/d/x");
        assert_eq!(convert_to_mingw("/c/x"), "/c/x");
    }

    #[test]
    #[cfg(windows)]
    fn windows_bash_script_uses_mingw_paths() {
        assert_eq!(
            script_path(Kind::Bash, r"C:\Users\me\.vmr", r"C:\Users\me"),
            "$HOME/.vmr"
        );
        assert_eq!(
            script_path(Kind::Bash, r"C:\Users\me\.vmr", "/c/Users/me"),
            "$HOME/.vmr"
        );
        assert_eq!(
            script_path(Kind::Bash, r"D:\tools", r"C:\Users\me"),
            "/d/tools"
        );
        // An unrelated home prefix must not be stripped mid-component.
        assert_eq!(
            script_path(Kind::Bash, r"C:\Users\meg\.vmr", r"C:\Users\me"),
            "/c/Users/meg/.vmr"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn strip_hook_keeps_dynamic_env_lines() {
        let dir = std::env::temp_dir().join(format!("vmr-shell-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("vmr.sh");
        fs::write(
            &f,
            "# cd hook start\nexport PATH=\"$HOME/.vmr\":\"${PATH}\"\ncdhook() { :; }\n# cd hook end\nexport GOROOT=\"$HOME/go\"\n",
        )
        .unwrap();
        let mut sh = Shell::new(Kind::Bash);
        sh.home = "/home/u".to_string();
        sh.strip_hook_section(&f, "/home/u/.vmr");
        let content = fs::read_to_string(&f).unwrap();
        assert!(!content.contains("cd hook"), "{content}");
        assert!(!content.contains("cdhook"), "{content}");
        assert!(!content.contains(".vmr"), "{content}");
        assert!(content.contains("GOROOT"), "{content}");
        // Idempotent.
        sh.strip_hook_section(&f, "/home/u/.vmr");
        assert_eq!(fs::read_to_string(&f).unwrap(), content);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pack_lines_shape() {
        let sh = Shell::new(Kind::Bash);
        assert_eq!(sh.pack_path("/a/b"), "export PATH=\"/a/b\":\"${PATH}\"");
        assert_eq!(
            sh.pack_env("GOROOT", "$HOME/go"),
            "export GOROOT=\"$HOME/go\""
        );
        assert_eq!(sh.pack_env("X", ""), "export X=");
        let fsh = Shell::new(Kind::Fish);
        assert_eq!(fsh.pack_path("/a/b"), "fish_add_path --global \"/a/b\"");
        assert_eq!(fsh.pack_env("X", ""), "set --global X ");
    }

    #[test]
    fn init_script_guards_vm_disable_and_defines_svmr() {
        let sh = Shell::new(Kind::Bash);
        let s = sh.init_script();
        assert!(s.contains("if [ -z \"$VM_DISABLE\" ]"), "{s}");
        assert!(s.contains(". \"$HOME/.vmr/vmr.sh\""), "{s}");
        assert!(s.contains("cdhook() {"), "{s}");
        assert!(s.contains("alias cd='cdhook'"), "{s}");
        assert!(!s.contains("alias|grep cdhook"), "{s}");
        assert!(s.contains("export VMR_CD_INIT"), "{s}");
        assert!(s.contains("VMR_CD_CACHE"), "{s}");
        assert!(s.contains("cd_hook.cache"), "{s}");
        assert!(s.contains("vmr cdhook"), "{s}");
        assert!(s.contains("VMR_SESSION_PROJECT"), "{s}");
        assert!(s.contains("VMR_SESSION_ACTIVE"), "{s}");
        assert!(s.contains("session_exit"), "{s}");
        assert!(!s.contains("__VMR_"), "{s}");
        assert!(s.contains("alias svmr='export VM_DISABLE=\"\""), "{s}");
        #[cfg(not(windows))]
        {
            assert!(s.contains("_vmr_refresh() {"), "{s}");
            assert!(s.contains("vmr() {"), "{s}");
            assert!(s.contains("command vmr \"$@\""), "{s}");
            assert!(s.contains("use|u|h|uninstall|uni|r"), "{s}");
            assert!(s.contains("VMR_SDK_INSTALLATION_DIR"), "{s}");
            assert!(s.contains("env_keys.txt"), "{s}");
            // The install directory is added by _vmr_refresh, not twice by the cd hook.
            assert!(!s.contains("export PATH=\"$HOME/.vmr\""), "{s}");
        }

        let fish = Shell::new(Kind::Fish).init_script();
        assert!(fish.contains("if not set -q VM_DISABLE"), "{fish}");
        assert!(fish.contains("source \"$HOME/.vmr/vmr.fish\""), "{fish}");
        assert!(fish.contains("_vmr_cdhook"), "{fish}");
        assert!(fish.contains("set -gx VMR_CD_INIT"), "{fish}");
        assert!(fish.contains("VMR_CD_CACHE"), "{fish}");
        assert!(fish.contains("cd_hook.cache"), "{fish}");
        assert!(fish.contains("VMR_SESSION_PROJECT"), "{fish}");
        assert!(fish.contains("session_exit"), "{fish}");
        assert!(!fish.contains("__VMR_"), "{fish}");
        assert!(fish.contains("function svmr"), "{fish}");
        #[cfg(not(windows))]
        {
            assert!(fish.contains("function _vmr_refresh"), "{fish}");
            assert!(fish.contains("function vmr"), "{fish}");
            assert!(fish.contains("command vmr $argv"), "{fish}");
            assert!(fish.contains("case use u h uninstall uni r"), "{fish}");
        }

        let ps = Shell::new(Kind::PowerShell).init_script();
        assert!(ps.contains("$env:VM_DISABLE"), "{ps}");
        assert!(ps.contains("function cdhook"), "{ps}");
        assert!(ps.contains("Set-Alias -Name cd"), "{ps}");
        assert!(ps.contains("VMR_CD_INIT"), "{ps}");
        assert!(ps.contains("VMR_CD_CACHE"), "{ps}");
        assert!(ps.contains("cd_hook.cache"), "{ps}");
        assert!(ps.contains("VMR_SESSION_PROJECT"), "{ps}");
        assert!(ps.contains("session_exit"), "{ps}");
        assert!(!ps.contains("__VMR_"), "{ps}");
        assert!(ps.contains("function global:svmr"), "{ps}");
        assert!(ps.contains("function global:vmrsource"), "{ps}");
        assert!(ps.contains("function global:vmr "), "{ps}");
        assert!(ps.contains("env_keys.txt"), "{ps}");
        assert!(
            ps.contains("'use', 'u', 'h', 'uninstall', 'uni', 'r'"),
            "{ps}"
        );
        assert!(!ps.contains("vmr.sh"), "{ps}");
    }

    /// Runs the generated bash hook against a stub `vmr` and checks that the project lookup
    /// is skipped until the shell leaves the cached project.
    #[test]
    #[cfg(unix)]
    fn cd_hook_reuses_the_cached_project() {
        use std::process::Command;

        if Command::new("bash").arg("-c").arg(":").status().is_err() {
            return; // no bash on this host; nothing to exercise
        }
        let root = std::env::temp_dir().join(format!("vmr-cd-hook-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let project = root.join("project");
        let nested = project.join("nested");
        let deep = nested.join("deep");
        fs::create_dir_all(&deep).unwrap();
        fs::write(project.join(".vmr.lock"), r#"{"go":"1.22.1"}"#).unwrap();
        fs::write(nested.join(".vmr.lock"), r#"{"go":"1.21.0"}"#).unwrap();

        let cache = root.join("cd_hook.cache");
        let exit = root.join("session_exit");
        let calls = root.join("calls");
        let hook = scripts::HOOK_SH
            .replace("__VMR_CD_CACHE__", &cache.to_string_lossy())
            .replace("__VMR_SESSION_EXIT__", &exit.to_string_lossy());

        // The stub records every call and mirrors `vmr cdhook` by caching the nearest lock
        // directory above the working directory.
        const TEMPLATE: &str = r#"
VMR_CD_INIT=
VMR_CD_CACHE=
vmr() {
    printf 'call\n' >> "@CALLS@"
    local d="$PWD" found=
    while :; do
        if [ -f "$d/.vmr.lock" ]; then found="$d"; break; fi
        [ "$d" = "/" ] && break
        d="${d%/*}"
        [ -z "$d" ] && d=/
    done
    printf '%s\n' "$found" > "@CACHE@"
}
@HOOK@
cdhook "@PROJECT@"
cdhook "@DEEP@"
cdhook "@NESTED@"
cdhook "@PROJECT@"
cdhook "@ROOT@"
cdhook "@PROJECT@"
# `vmr doctor` runs as a child process, so the marker must be exported.
[ -n "${VMR_CD_INIT}" ] || { echo "VMR_CD_INIT is not set" >&2; exit 1; }
sh -c '[ -n "$VMR_CD_INIT" ]' || { echo "VMR_CD_INIT is not exported" >&2; exit 1; }
"#;
        let script = TEMPLATE
            .replace("@CALLS@", &calls.to_string_lossy())
            .replace("@CACHE@", &cache.to_string_lossy())
            .replace("@HOOK@", &hook)
            .replace("@PROJECT@", &project.to_string_lossy())
            .replace("@DEEP@", &deep.to_string_lossy())
            .replace("@NESTED@", &nested.to_string_lossy())
            .replace("@ROOT@", &root.to_string_lossy());

        let status = Command::new("bash")
            .arg("-c")
            .arg(&script)
            .current_dir(&root)
            .status()
            .expect("bash starts");
        assert!(status.success());

        // Initial check at `root` (1), project (2), deep is inside the cached project (skip),
        // nested has its own lock (3), project (4), root (5), project again (6).
        let invocations = fs::read_to_string(&calls)
            .unwrap_or_default()
            .lines()
            .count();
        assert_eq!(invocations, 6, "expected 6 vmr cdhook invocations");
        let _ = fs::remove_dir_all(&root);
    }

    /// The session hook must keep the child shell alive inside the project and end it when
    /// `cd` leaves, recording the destination for the outer hook.
    #[test]
    #[cfg(unix)]
    fn lock_session_hook_leaves_the_project() {
        use std::process::Command;

        if Command::new("bash").arg("-c").arg(":").status().is_err() {
            return;
        }
        let root = std::env::temp_dir().join(format!("vmr-session-leave-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("project/sub")).unwrap();
        fs::create_dir_all(root.join("outside")).unwrap();
        fs::write(root.join("project/.vmr.lock"), r#"{"go":"1.22.1"}"#).unwrap();
        let root = fs::canonicalize(&root).unwrap();
        let project = root.join("project");
        let sub = project.join("sub");
        let outside = root.join("outside");
        let exit = root.join("session_exit");
        let hook =
            scripts::SESSION_HOOK_SH.replace("__VMR_SESSION_EXIT__", &exit.to_string_lossy());
        let script = format!(
            "VM_DISABLE=111\nVMR_SESSION_PROJECT='{project}'\n{hook}\n\
             cdhook '{sub}'\nprintf 'inside\\n'\n\
             cdhook '{outside}'\nprintf 'after\\n'\n",
            project = project.display(),
            sub = sub.display(),
            outside = outside.display(),
        );
        let out = Command::new("bash")
            .arg("-c")
            .arg(&script)
            .current_dir(&root)
            .output()
            .expect("bash starts");
        assert!(
            out.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        // The shell exited at the second cd, before reaching the trailing printf.
        assert_eq!(String::from_utf8_lossy(&out.stdout), "inside\n");
        assert_eq!(
            fs::read_to_string(&exit).unwrap().trim(),
            outside.to_str().unwrap()
        );

        // A subshell inheriting VMR_SESSION_ACTIVE must not install a second hook: only the
        // shell directly started by vmr can end the session.
        fs::remove_file(&exit).unwrap();
        let subshell = format!(
            "shopt -s expand_aliases\nVM_DISABLE=111\nVMR_SESSION_PROJECT='{project}'\n\
             VMR_SESSION_ACTIVE=1\n{hook}\neval \"cd -- '{outside}'\"\nprintf 'survived\\n'\n",
            project = project.display(),
            outside = outside.display(),
        );
        let out = Command::new("bash")
            .arg("-c")
            .arg(&subshell)
            .current_dir(&root)
            .output()
            .expect("bash starts");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout), "survived\n");
        assert!(!exit.exists(), "a subshell must not write the exit file");
        let _ = fs::remove_dir_all(&root);
    }

    /// After a session ends, the outer hook must move to the recorded directory before the
    /// next lookup, otherwise the user lands back at the project root.
    #[test]
    #[cfg(unix)]
    fn cd_hook_follows_the_session_exit_destination() {
        use std::process::Command;

        if Command::new("bash").arg("-c").arg(":").status().is_err() {
            return;
        }
        let root = std::env::temp_dir().join(format!("vmr-session-follow-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("project")).unwrap();
        fs::create_dir_all(root.join("outside")).unwrap();
        let root = fs::canonicalize(&root).unwrap();
        let project = root.join("project");
        let outside = root.join("outside");
        let cache = root.join("cd_hook.cache");
        let exit = root.join("session_exit");
        let calls = root.join("calls");
        let hook = scripts::HOOK_SH
            .replace("__VMR_CD_CACHE__", &cache.to_string_lossy())
            .replace("__VMR_SESSION_EXIT__", &exit.to_string_lossy());
        let script = format!(
            r#"
VMR_CD_INIT=1
VMR_CD_CACHE=
vmr() {{
    printf '%s\n' "$PWD" >> '{calls}'
    if [ "$PWD" = '{project}' ]; then printf '%s\n' '{outside}' > '{exit}'; fi
    printf '\n' > '{cache}'
}}
{hook}
cdhook '{project}'
printf 'pwd=%s\n' "$PWD"
"#,
            calls = calls.display(),
            project = project.display(),
            outside = outside.display(),
            exit = exit.display(),
            cache = cache.display(),
        );
        let out = Command::new("bash")
            .arg("-c")
            .arg(&script)
            .current_dir(&root)
            .output()
            .expect("bash starts");
        assert!(
            out.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(&format!("pwd={}", outside.display())),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let observed = fs::read_to_string(&calls).unwrap();
        assert_eq!(
            observed.lines().collect::<Vec<_>>(),
            vec![project.to_str().unwrap(), outside.to_str().unwrap()]
        );
        assert!(!exit.exists(), "the exit file must be consumed");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn init_line_is_shell_specific() {
        assert_eq!(
            Shell::new(Kind::Zsh).init_line(),
            "eval \"$(vmr init zsh)\""
        );
        assert_eq!(Shell::new(Kind::Fish).init_line(), "vmr init fish | source");
        assert!(
            Shell::new(Kind::PowerShell)
                .init_line()
                .contains("vmr init powershell")
        );
    }

    #[test]
    fn init_line_migrates_legacy_blocks_idempotently() {
        let dir = std::env::temp_dir().join(format!("vmr-shell-init-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let rc = dir.join(".zshrc");
        fs::write(
            &rc,
            "export A=1\n# vm_envs start\nif [ -z \"$VM_DISABLE\" ]; then\n    . ~/.vmr/vmr.sh\nfi\n# vm_envs end\n\n# vmr source alias start\nalias svmr=\"export VM_DISABLE='' && source /home/u/.zshrc\"\n# vmr source alias end\n",
        )
        .unwrap();
        let line = "eval \"$(vmr init zsh)\"";
        ensure_init_line(&rc, line);
        let once = fs::read_to_string(&rc).unwrap();
        assert!(once.contains("export A=1"));
        assert!(!once.contains("# vm_envs start"));
        assert!(!once.contains("# vmr source alias start"));
        assert_eq!(once.matches(line).count(), 1);
        ensure_init_line(&rc, line);
        assert_eq!(fs::read_to_string(&rc).unwrap(), once);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(not(windows))]
    fn env_key_list_appends_once() {
        assert_eq!(merge_env_key("", "GOROOT"), "GOROOT\n");
        assert_eq!(merge_env_key("GOROOT\n", "GOROOT"), "GOROOT\n");
        assert_eq!(
            merge_env_key("GOROOT\n", "JAVA_HOME"),
            "GOROOT\nJAVA_HOME\n"
        );
        // A list without a trailing newline still gets one separator.
        assert_eq!(merge_env_key("GOROOT", "JAVA_HOME"), "GOROOT\nJAVA_HOME\n");
    }
}
