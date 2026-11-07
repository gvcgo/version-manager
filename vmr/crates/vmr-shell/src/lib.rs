//! vmr-shell: shell hook environment injection and automatic cd-hook switching (plan.md §3.8, requirement 7).
//!
//! Mirrors Go `internal/shell/`: a shared scheme for bash/zsh/fish —
//! - Shell environment file `~/.vmr/vmr.sh` (bash/zsh) or `~/.vmr/vmr.fish` (fish);
//! - `update_vmr_shell_file`: idempotent replacement between `# cd hook start … # cd hook end` markers;
//! - rc files (`.bashrc`/`.zshrc`/`config.fish`) get a source block appended (`VM_DISABLE` guard);
//! - `PackPath`/`PackEnv` line injection / prefix-based removal (Set/Unset).
//!
//! Session contract: skip injection when `VM_DISABLE=111`; the cd hook calls `vmr cdhook` for automatic switching.
//! On Windows, the generated PowerShell config includes the cd hook plus `vmrsource` and a
//! `vmr` wrapper: the registry broadcast only reaches newly started processes, and a child
//! process cannot change its parent's environment, so the wrapper re-reads
//! `HKCU\Environment` after `vmr use` / `vmr uninstall` to refresh the current window.

use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::paths;

#[cfg(windows)]
mod win_patch;
#[cfg(windows)]
mod win_reg;

pub const VM_DISABLE_ENV_NAME: &str = "VM_DISABLE";
pub const VM_CD_INIT_ENV_NAME: &str = "VMR_CD_INIT";
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
        match self {
            Kind::Fish => format!(
                "fish_add_path --global \"{inst}\"

function _vmr_cdhook --on-variable=\"PWD\" --description \"version manager cd hook\"
    if type -q vmr
        vmr cdhook
    end
end

if set -q \"$VMR_CD_INIT\"
    set VMR_CD_INIT \"vmr_cd_init\"
    cd \"$(pwd)\"
end"
            ),
            Kind::PowerShell => String::from(
                "function cdhook {
    if (Test-Path $args[0]) {
        chdir $args[0]
        vmr cdhook
    }
}

Set-Alias -Name cd -Option AllScope -Value cdhook

if (\"\" -eq \"$env:VMR_CD_INIT\") {
    $env:VMR_CD_INIT = \"vmr_cd_init\"
    cdhook (Get-Location).Path
}",
            ),
            _ => format!(
                "export PATH=\"{inst}\":\"${{PATH}}\"

cdhook() {{
    if [ $# -eq 0 ]; then
        builtin cd
    else
        builtin cd \"$@\" && vmr cdhook
    fi
}}
alias cd='cdhook'

if [ -z \"${{VMR_CD_INIT}}\" ]; then
    VMR_CD_INIT=\"vmr_cd_init\"
    cdhook \"$(pwd)\"
fi"
            ),
        }
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

/// Quotes a value for a PowerShell single-quoted literal.
fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Re-reads the vmr-managed part of the Windows user environment into the current
/// PowerShell process.
///
/// A child process cannot change its parent's environment, so `vmr use` can only write
/// `HKCU\Environment`; this block is evaluated from the profile and is what makes the
/// current window see the new SDK. It replaces only entries under the versions directory
/// (or the `%VMR_VERSIONS%` registry form) and keeps unrelated PATH entries, so developer
/// shells that added their own directories are not reset.
const POWERSHELL_REFRESH: &str = r#"function global:vmrsource {
    $versions = [System.Environment]::GetEnvironmentVariable('VMR_VERSIONS', 'User')
    if (-not $versions) { $versions = __VMR_VERSIONS_DIR__ }
    if ($versions) { $env:VMR_VERSIONS = $versions }
    $isVmPath = {
        param($entry)
        if (-not $entry) { return $false }
        $p = $entry.Replace('/', '\').TrimEnd('\')
        if ($p.StartsWith('%VMR_VERSIONS%', [System.StringComparison]::OrdinalIgnoreCase)) { return $true }
        if (-not $versions) { return $false }
        $v = $versions.Replace('/', '\').TrimEnd('\')
        return ($p -ieq $v) -or $p.StartsWith($v + '\', [System.StringComparison]::OrdinalIgnoreCase)
    }
    $userPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
    $vmrEntries = @()
    if ($userPath) {
        foreach ($entry in $userPath.Split(';')) {
            if (-not (& $isVmPath $entry)) { continue }
            $expanded = $entry
            if ($expanded.StartsWith('%VMR_VERSIONS%') -and $versions) {
                $expanded = $versions + $expanded.Substring(('%VMR_VERSIONS%').Length)
            }
            if (-not ($vmrEntries -contains $expanded)) { $vmrEntries += $expanded }
        }
    }
    $current = $env:Path
    if (-not $current) { $current = '' }
    $kept = @()
    foreach ($entry in $current.Split(';')) {
        if ($entry -and -not (& $isVmPath $entry)) { $kept += $entry }
    }
    $env:Path = (@($vmrEntries) + $kept) -join ';'
    $keyFile = __VMR_ENV_KEYS__
    if (Test-Path -LiteralPath $keyFile) {
        foreach ($key in @(Get-Content -LiteralPath $keyFile)) {
            $key = $key.Trim()
            if (-not $key -or $key -ieq 'Path') { continue }
            $value = [System.Environment]::GetEnvironmentVariable($key, 'User')
            if ($null -ne $value) {
                Set-Item -Path ('Env:' + $key) -Value $value
            } else {
                Remove-Item -Path ('Env:' + $key) -ErrorAction SilentlyContinue
            }
        }
    }
}
$global:__vmr_exe = $null
$vmrCommand = Get-Command -Name 'vmr' -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($vmrCommand) { $global:__vmr_exe = $vmrCommand.Source }
if (-not $global:__vmr_exe) {
    $vmrFallback = Join-Path $HOME '.vmr\vmr.exe'
    if (Test-Path -LiteralPath $vmrFallback) { $global:__vmr_exe = $vmrFallback }
}
if ($global:__vmr_exe) {
    function global:vmr {
        & $global:__vmr_exe @args
        $vmrExit = $LASTEXITCODE
        if ($vmrExit -eq 0 -and $args.Count -gt 0 -and (@('use', 'u', 'h', 'uninstall', 'uni', 'r') -contains [string]$args[0])) {
            vmrsource
        }
        $global:LASTEXITCODE = $vmrExit
    }
}
"#;

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

    /// Path of the vmr shell environment file (`~/.vmr/vmr.sh|vmr.fish`).
    pub fn vm_env_conf_path(&self) -> PathBuf {
        paths::work_dir().join(self.kind.env_file_name())
    }

    #[cfg(not(windows))]
    fn env_file_text(&self) -> String {
        fs::read_to_string(self.vm_env_conf_path()).unwrap_or_default()
    }

    #[cfg(not(windows))]
    fn write_env_file(&self, data: &str) {
        let _ = fs::write(self.vm_env_conf_path(), data.trim_start_matches('\n'));
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
    pub fn init_script(&self) -> String {
        let env_file = script_path(
            self.kind,
            &self.vm_env_conf_path().to_string_lossy(),
            &self.home,
        );
        let conf = script_path(self.kind, &self.conf_path().to_string_lossy(), &self.home);
        let install = paths::work_dir().to_string_lossy().into_owned();
        let hook = indent_block(&self.kind.hook_script(&install, &self.home), "    ");
        let refresh = POWERSHELL_REFRESH
            .replace(
                "__VMR_VERSIONS_DIR__",
                &ps_quote(&paths::versions_dir().to_string_lossy()),
            )
            .replace(
                "__VMR_ENV_KEYS__",
                &ps_quote(&paths::work_dir().join("env_keys.txt").to_string_lossy()),
            );
        let refresh = indent_block(&refresh, "    ");
        match self.kind {
            Kind::Fish => format!(
                "if not set -q {VM_DISABLE_ENV_NAME}\n    if test -f \"{env_file}\"\n        . \"{env_file}\"\n    end\n{hook}\nend\n\nfunction svmr --description \"leave a vmr session and reload the shell config\"\n    set -gx {VM_DISABLE_ENV_NAME} ''\n    source \"{conf}\"\nend\n"
            ),
            Kind::PowerShell => format!(
                "if (-not $env:{VM_DISABLE_ENV_NAME}) {{\n    $env:Path = (Join-Path $HOME '.vmr') + ';' + $env:Path\n{hook}\n{refresh}\n}}\nfunction global:svmr {{ $env:{VM_DISABLE_ENV_NAME} = ''; . $PROFILE }}\n"
            ),
            _ => format!(
                "if [ -z \"${VM_DISABLE_ENV_NAME}\" ]; then\n    [ -f \"{env_file}\" ] && . \"{env_file}\"\n{hook}\nfi\nalias svmr='export {VM_DISABLE_ENV_NAME}=\"\" && source \"{conf}\"'\n"
            ),
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
        assert!(s.contains("VMR_CD_INIT"), "{s}");
        assert!(s.contains("alias svmr='export VM_DISABLE=\"\""), "{s}");

        let fish = Shell::new(Kind::Fish).init_script();
        assert!(fish.contains("if not set -q VM_DISABLE"), "{fish}");
        assert!(fish.contains(". \"$HOME/.vmr/vmr.fish\""), "{fish}");
        assert!(fish.contains("_vmr_cdhook"), "{fish}");
        assert!(fish.contains("function svmr"), "{fish}");

        let ps = Shell::new(Kind::PowerShell).init_script();
        assert!(ps.contains("$env:VM_DISABLE"), "{ps}");
        assert!(ps.contains("function cdhook"), "{ps}");
        assert!(ps.contains("Set-Alias -Name cd"), "{ps}");
        assert!(ps.contains("VMR_CD_INIT"), "{ps}");
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
}
