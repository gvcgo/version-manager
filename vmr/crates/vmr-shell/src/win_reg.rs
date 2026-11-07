//! Windows environment backend: writes `HKCU\Environment` and broadcasts
//! `WM_SETTINGCHANGE` (mirrors Go `internal/shell/win.go`).
//!
//! `PATH` is stored as an expand-string (`REG_EXPAND_SZ`) and new entries are prepended;
//! `UnsetPath` removes every occurrence. Other variables are plain string values.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, REG_EXPAND_SZ};
use winreg::{RegKey, RegValue};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};

const ENV_KEY: &str = "Environment";
const PATH_VALUE: &str = "Path";

fn env_key() -> Option<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(ENV_KEY, KEY_ALL_ACCESS)
        .ok()
}

fn is_path(key: &str) -> bool {
    key.eq_ignore_ascii_case(PATH_VALUE)
}

/// Windows PATH entries are case-insensitive, accept `/` or `\`, and may be stored in the
/// `%VMR_VERSIONS%\...` expand-string form; compare all of those as the same entry.
fn path_eq(a: &str, b: &str) -> bool {
    fn canonical(p: &str) -> String {
        crate::win_patch::expand_versions(p)
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    }
    canonical(a) == canonical(b)
}

/// Lists the additional (non-PATH) variables vmr has written so the PowerShell profile can
/// refresh the current window without touching variables it does not own. Keys removed
/// from the registry stay listed so a refresh also clears them from `$env:`.
fn env_keys_path() -> std::path::PathBuf {
    vmr_core::paths::work_dir().join("env_keys.txt")
}

fn remember_env_key(key: &str) {
    let path = env_keys_path();
    let data = std::fs::read_to_string(&path).unwrap_or_default();
    if data.lines().any(|l| l.trim().eq_ignore_ascii_case(key)) {
        return;
    }
    let mut out = data;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(key);
    out.push('\n');
    let _ = std::fs::write(&path, out);
}

pub fn set_env(key: &str, value: &str) {
    if is_path(key) {
        return;
    }
    let Some(k) = env_key() else { return };
    if k.set_value(key, &value).is_ok() {
        remember_env_key(key);
        broadcast();
    }
}

pub fn unset_env(key: &str) {
    if is_path(key) {
        return;
    }
    let Some(k) = env_key() else { return };
    if k.delete_value(key).is_ok() {
        broadcast();
    }
}

/// Reads a registry value as a string (used for the `VMR_VERSIONS` check).
pub fn get_env(key: &str) -> Option<String> {
    env_key()?.get_value::<String, _>(key).ok()
}

pub fn set_path(path: &str) {
    let path = crate::win_patch::tidy_path(path);
    let Some(k) = env_key() else { return };
    let old: String = k.get_value(PATH_VALUE).unwrap_or_default();
    if old.split(';').any(|p| path_eq(p, &path)) {
        return;
    }
    let new = if old.is_empty() {
        path.to_string()
    } else {
        format!("{path};{old}")
    };
    if write_expand(&k, &new).is_ok() {
        broadcast();
    }
}

pub fn unset_path(path: &str) {
    let path = crate::win_patch::untidy_path(path);
    let Some(k) = env_key() else { return };
    let old: String = k.get_value(PATH_VALUE).unwrap_or_default();
    let new = old
        .split(';')
        .filter(|p| !path_eq(p, &path))
        .collect::<Vec<_>>()
        .join(";");
    if new != old && write_expand(&k, &new).is_ok() {
        broadcast();
    }
}

fn write_expand(k: &RegKey, value: &str) -> std::io::Result<()> {
    let units: Vec<u16> = OsStr::new(value).encode_wide().chain([0]).collect();
    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
    k.set_raw_value(
        PATH_VALUE,
        &RegValue {
            bytes,
            vtype: REG_EXPAND_SZ,
        },
    )
}

fn broadcast() {
    let name: Vec<u16> = OsStr::new(ENV_KEY).encode_wide().chain([0]).collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            name.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }
}
