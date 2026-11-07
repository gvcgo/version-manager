//! Shell script bodies embedded at compile time from `../scripts/`.
//!
//! Templates use `__VMR_*__` placeholders that [`crate::fill`] substitutes with
//! runtime paths (env file, keys file, versions dir, rc file) or with blocks assembled
//! from other templates (refresh/wrapper/hook) before the init script is printed.

/// bash/zsh cd hook, emitted by `vmr init` and evaluated from the rc file.
pub(crate) const HOOK_SH: &str = include_str!("../scripts/hook.sh");

/// fish cd hook (`--on-variable=PWD`).
pub(crate) const HOOK_FISH: &str = include_str!("../scripts/hook.fish");

/// PowerShell cd hook (`cdhook` + `Set-Alias cd`, first-start `VMR_CD_INIT`).
pub(crate) const HOOK_POWERSHELL: &str = include_str!("../scripts/hook.ps1");

/// git-bash cd hook with the marked block written into `%USERPROFILE%\.bashrc`.
#[cfg(windows)]
pub(crate) const HOOK_GITBASH: &str = include_str!("../scripts/hook-gitbash.sh");

/// bash/zsh refresh used by the `vmr` wrapper (and by the integration itself).
///
/// Drops every vmr-managed PATH entry (`~/.vmr` plus the versions directory) and unsets the
/// variables recorded in `env_keys.txt`, then re-sources the env file. Filtering first keeps
/// repeated refreshes from stacking duplicate PATH entries; unsetting first makes variables
/// that disappeared from the file (uninstall) disappear from the shell too.
#[cfg(not(windows))]
pub(crate) const REFRESH_SH: &str = include_str!("../scripts/refresh.sh");

/// bash/zsh `vmr` wrapper: run the binary, then refresh the current shell after commands
/// that change the global environment.
#[cfg(not(windows))]
pub(crate) const WRAPPER_SH: &str = include_str!("../scripts/wrapper.sh");

/// fish refresh (PATH dedupe uses the list form; `fish_add_path` in the env file is already
/// idempotent, so only stale entries are filtered out here).
#[cfg(not(windows))]
pub(crate) const REFRESH_FISH: &str = include_str!("../scripts/refresh.fish");

/// fish `vmr` wrapper (same contract as [`WRAPPER_SH`]).
#[cfg(not(windows))]
pub(crate) const WRAPPER_FISH: &str = include_str!("../scripts/wrapper.fish");

/// Re-reads the vmr-managed part of the Windows user environment into the current
/// PowerShell process.
///
/// A child process cannot change its parent's environment, so `vmr use` can only write
/// `HKCU\Environment`; this block is evaluated from the profile and is what makes the
/// current window see the new SDK. It replaces only entries under the versions directory
/// (or the `%VMR_VERSIONS%` registry form) and keeps unrelated PATH entries, so developer
/// shells that added their own directories are not reset.
pub(crate) const REFRESH_POWERSHELL: &str = include_str!("../scripts/refresh.ps1");

/// Unix bash/zsh integration printed by `vmr init <shell>`.
#[cfg(not(windows))]
pub(crate) const INIT_SH: &str = include_str!("../scripts/init.sh");

/// Windows git-bash integration (no env-file refresh; the hook carries the PATH line).
#[cfg(windows)]
pub(crate) const INIT_SH_WINDOWS: &str = include_str!("../scripts/init-windows.sh");

/// Unix fish integration printed by `vmr init fish`.
#[cfg(not(windows))]
pub(crate) const INIT_FISH: &str = include_str!("../scripts/init.fish");

/// Windows fish integration (no env-file refresh).
#[cfg(windows)]
pub(crate) const INIT_FISH_WINDOWS: &str = include_str!("../scripts/init-windows.fish");

/// PowerShell integration printed by `vmr init powershell`.
pub(crate) const INIT_POWERSHELL: &str = include_str!("../scripts/init.ps1");

/// Tail appended to the clap-generated zsh completion script.
///
/// zsh `compdef` is deferred: rc files may evaluate `vmr init` before `compinit` (or never
/// run `compinit` at all), so registration happens on the first prompt instead of silently
/// being skipped. If no completion system is loaded by then, `compinit` is initialized.
pub const ZSH_COMPLETION_TAIL: &str = include_str!("../scripts/completion.zsh");
