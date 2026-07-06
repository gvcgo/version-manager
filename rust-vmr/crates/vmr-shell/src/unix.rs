use std::env;
use std::fs;

use crate::common::{self, BashShell, FishShell, Sheller, ZshShell};

/// Shell wraps a concrete shell implementation detected from the `SHELL`
/// environment variable.
pub struct Shell {
    inner: Box<dyn Sheller>,
}

/// Detect the current user's shell from the `SHELL` env var and return a
/// matching `Shell`.  Falls back to `BashShell` when detection fails.
pub fn new_shell() -> Shell {
    let shell_path = env::var("SHELL").unwrap_or_default();
    let inner: Box<dyn Sheller> = if shell_path.ends_with(common::BASH) {
        Box::new(BashShell)
    } else if shell_path.ends_with(common::ZSH) {
        Box::new(ZshShell)
    } else if shell_path.ends_with(common::FISH) {
        Box::new(FishShell)
    } else {
        Box::new(BashShell)
    };
    Shell { inner }
}

impl Shell {
    /// Append the packed `path` to the VM env file (unless already present).
    pub fn set_path(&self, path: &str) {
        let conf = self.inner.vm_env_conf_path();
        let data = fs::read_to_string(&conf).unwrap_or_default().trim().to_string();

        let packed = self.inner.pack_path(path);
        if !data.contains(&packed) {
            let new_data = if data.is_empty() {
                packed
            } else {
                format!("{}\n{}", data, &packed)
            };
            let _ = fs::write(&conf, new_data);
        }
    }

    /// Remove the packed `path` from the VM env file.
    pub fn unset_path(&self, path: &str) {
        let conf = self.inner.vm_env_conf_path();
        let data = fs::read_to_string(&conf).unwrap_or_default().trim().to_string();

        let packed = self.inner.pack_path(path);
        if data.contains(&packed) {
            let new_data = data
                .replace(&packed, "")
                .replace("\n\n", "\n")
                .trim()
                .to_string();
            let _ = fs::write(&conf, new_data);
        }
    }

    /// Append `key=value` (packed by the current shell) to the VM env file
    /// unless it already exists.
    pub fn set_env(&self, key: &str, value: &str) {
        let conf = self.inner.vm_env_conf_path();
        let data = fs::read_to_string(&conf).unwrap_or_default();

        let packed = self.inner.pack_env(key, value);
        if !data.contains(&packed) {
            let new_data = format!("{}\n{}", data.trim_end(), &packed);
            let _ = fs::write(&conf, new_data);
        }
    }

    /// Remove every line that starts with the packed `key`-only prefix from
    /// the VM env file.
    pub fn unset_env(&self, key: &str) {
        let conf = self.inner.vm_env_conf_path();
        let data = fs::read_to_string(&conf).unwrap_or_default();

        let prefix = self.inner.pack_env(key, "");
        let mut lines: Vec<&str> = data.lines().collect();
        lines.retain(|line| !line.starts_with(&prefix));
        let new_data = lines.join("\n").replace("\n\n", "\n").trim().to_string();
        let _ = fs::write(&conf, new_data);
    }
}
