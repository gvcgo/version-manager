//! Plugin lifecycle (mirrors Go `internal/luapi/plugin/{plugin.go,plugins.go,fromlua.go}`).
//!
//! Semantic highlights:
//! - Metadata (sdk_name/plugin_name/plugin_version/homepage/prequisite) is read from
//!   global strings; if the value is a function it is called for its return value (Go `GetLuaConfItemString`).
//! - Load validation: plugin_name/sdk_name/homepage/ic/crawl must all be present.
//! - Version retrieval: cache first (`<cache>/<plugin>/<plugin>.versions.json`, fresh within the retention
//!   period and cache not disabled); otherwise run `crawl()` live and filter by the current os/arch
//!   (keeping the **last** matching Item, mirroring Go), then write the cache.
//! - Install callbacks: when the `install`/`postInstall` global functions exist they are called with arguments, and
//!   their returned string must be "true" to count as success (Go `getFuncFromLua` semantics).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use mlua::Lua;

use crate::bindings::new_runtime;
use crate::installer_conf::ic_from_global;
use crate::types::{InstallerConfig, Item, VersionList};
use crate::utils_bind::os_arch;
use crate::version::{filter_current_platform, vl_from_value};

pub const PLUGIN_NAME: &str = "plugin_name";
pub const SDK_NAME: &str = "sdk_name";
pub const PLUGIN_VERSION: &str = "plugin_version";
pub const PREQUISITE: &str = "prequisite";
pub const HOMEPAGE: &str = "homepage";
pub const CRAWL: &str = "crawl";
pub const POST_INSTALL: &str = "postInstall";
pub const CUSTOM_INSTALL: &str = "install";

/// A single plugin.
pub struct Plugin {
    file_name: Option<String>,
    file_content: Option<String>,
    lua: Option<Lua>,
    pub plugin_name: String,
    pub sdk_name: String,
    pub plugin_version: String,
    pub prequisite: String,
    pub homepage: String,
    /// Version table for the current platform: version → Item.
    versions: HashMap<String, Item>,
    loaded: bool,
}

impl Plugin {
    pub fn from_file(file_name: String) -> Self {
        Plugin {
            file_name: Some(file_name),
            file_content: None,
            lua: None,
            plugin_name: String::new(),
            sdk_name: String::new(),
            plugin_version: String::new(),
            prequisite: String::new(),
            homepage: String::new(),
            versions: HashMap::new(),
            loaded: false,
        }
    }

    pub fn from_content(content: String) -> Self {
        Plugin {
            file_name: None,
            file_content: Some(content),
            lua: None,
            plugin_name: String::new(),
            sdk_name: String::new(),
            plugin_version: String::new(),
            prequisite: String::new(),
            homepage: String::new(),
            versions: HashMap::new(),
            loaded: false,
        }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    fn exec(&mut self) -> Result<(), String> {
        if self.lua.is_none() {
            self.lua = Some(new_runtime().map_err(|e| e.to_string())?);
        }
        let lua = self.lua.as_ref().unwrap();
        // Shared helper modules (e.g. `_vmr_common.lua`) are loaded with `dofile`; expose the
        // absolute plugin directory to plugin code.
        let plugin_dir = vmr_core::paths::plugin_dir();
        let _ = lua
            .globals()
            .set("__vmr_plugin_dir", plugin_dir.to_string_lossy().to_string());
        if let Some(name) = &self.file_name {
            let path = vmr_core::paths::plugin_dir().join(name);
            let content = fs::read_to_string(&path)
                .map_err(|_| format!("plugin file not found: {}", path.display()))?;
            lua.load(&content)
                .exec()
                .map_err(|e| format!("failed to load plugin {name}: {e}"))
        } else if let Some(content) = &self.file_content {
            lua.load(content)
                .exec()
                .map_err(|e| format!("failed to load plugin: {e}"))
        } else {
            Err("no plugin source".to_string())
        }
    }

    fn global_str(&self, name: &str) -> Option<String> {
        let lua = self.lua.as_ref()?;
        let v: mlua::Value = lua.globals().get(name).ok()?;
        match v {
            mlua::Value::String(s) => Some(crate::req::lua_string_to_owned(&s)),
            mlua::Value::Function(f) => f.call::<String>(()).ok(),
            other => Some(crate::req::str_of(&other)),
        }
    }

    /// Loads the script and parses the metadata (mirrors Go `Load`).
    pub fn load(&mut self) -> Result<(), String> {
        if self.loaded {
            return Ok(());
        }
        self.exec()?;
        let lua = self.lua.as_ref().unwrap();
        let exists = |name: &str| -> bool {
            !matches!(lua.globals().get::<mlua::Value>(name), Ok(mlua::Value::Nil))
        };
        self.plugin_name = self.global_str(PLUGIN_NAME).unwrap_or_default();
        if self.plugin_name.is_empty() {
            return Err("plugin name not defined".to_string());
        }
        self.sdk_name = self.global_str(SDK_NAME).unwrap_or_default();
        if self.sdk_name.is_empty() {
            return Err("SDK name not defined".to_string());
        }
        self.homepage = self.global_str(HOMEPAGE).unwrap_or_default();
        if self.homepage.is_empty() {
            return Err("homepage not defined".to_string());
        }
        self.plugin_version = self.global_str(PLUGIN_VERSION).unwrap_or_default();
        self.prequisite = self.global_str(PREQUISITE).unwrap_or_default();
        if !exists("ic") {
            return Err("installer config not found".to_string());
        }
        if !exists(CRAWL) {
            return Err("Crawler<function crawl> not found".to_string());
        }
        self.loaded = true;
        Ok(())
    }

    fn cache_path(&self) -> PathBuf {
        vmr_core::paths::cache_dir()
            .join(&self.plugin_name)
            .join(format!("{}.versions.json", self.plugin_name))
    }

    fn ensure_cache_dir(&self) -> PathBuf {
        let dir = vmr_core::paths::cache_dir().join(&self.plugin_name);
        if dir.exists() && !dir.is_dir() {
            let _ = fs::remove_dir_all(&dir);
        }
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn cache_fresh(&self) -> bool {
        if vmr_core::conf::get_cache_disabled() {
            return false;
        }
        let p = self.cache_path();
        let Ok(meta) = fs::metadata(&p) else {
            return false;
        };
        let Ok(modified) = meta.modified() else {
            return false;
        };
        let Ok(age) = modified.elapsed() else {
            return false;
        };
        age.as_secs() < vmr_core::conf::get_cache_retention_time() as u64
    }

    fn load_cache(&mut self) {
        if !self.cache_fresh() {
            return;
        }
        if let Ok(content) = fs::read_to_string(self.cache_path()) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, Item>>(&content) {
                self.versions = map;
            }
        }
    }

    fn save_cache(&self) {
        if self.versions.is_empty() {
            return;
        }
        let p = self.cache_path();
        self.ensure_cache_dir();
        if let Ok(content) = serde_json::to_string_pretty(&self.versions) {
            if content.len() > 10 {
                let _ = fs::write(p, content);
            }
        }
    }

    /// Version table for the current platform (cache first; otherwise live crawl + filter + cache).
    pub fn get_sdk_versions(&mut self) -> Result<HashMap<String, Item>, String> {
        self.get_sdk_versions_cached(true)
    }

    /// Version table for the current platform; `use_cache = false` ignores the cached
    /// `<plugin>.versions.json`, crawls live and rewrites it.
    pub fn get_sdk_versions_cached(
        &mut self,
        use_cache: bool,
    ) -> Result<HashMap<String, Item>, String> {
        if self.lua.is_none() {
            self.loaded = false;
            self.load()?;
        }
        if use_cache {
            self.load_cache();
        } else {
            self.versions.clear();
        }
        if !self.versions.is_empty() {
            return Ok(self.versions.clone());
        }
        let vl = self.run_crawl()?;
        let (os, arch) = os_arch();
        self.versions = filter_current_platform(&vl, &os, &arch);
        self.save_cache();
        Ok(self.versions.clone())
    }

    fn run_crawl(&mut self) -> Result<VersionList, String> {
        let lua = self.lua.as_ref().ok_or("plugin not loaded")?;
        let crawl = lua.globals().get::<mlua::Function>(CRAWL).map_err(|_| {
            format!(
                "invalid plugin: missing crawl function: {}",
                self.plugin_name
            )
        })?;
        let result: mlua::Value = crawl
            .call(())
            .map_err(|e| format!("crawl failed for {}: {e}", self.plugin_name))?;
        vl_from_value(&result).ok_or_else(|| {
            format!(
                "crawl() did not return a version list: {}",
                self.plugin_name
            )
        })
    }

    /// Version names in descending order (vmr semantics).
    ///
    /// This legacy wrapper keeps swallowing crawl errors for callers that only need a best-effort
    /// list; the CLI uses [`Self::sorted_versions_result`] to report them.
    pub fn sorted_versions(&mut self) -> Vec<String> {
        self.sorted_versions_result().unwrap_or_default()
    }

    /// Version names in descending order, propagating plugin/crawl errors.
    pub fn sorted_versions_result(&mut self) -> Result<Vec<String>, String> {
        let versions = self.get_sdk_versions()?;
        let mut names: Vec<String> = versions.keys().cloned().collect();
        vmr_utils::semver::sort_versions(&mut names);
        Ok(names)
    }

    pub fn get_version(&mut self, name: &str) -> Option<Item> {
        self.get_version_result(name).ok().flatten()
    }

    /// Looks up a version, propagating plugin/crawl errors (a missing version is `Ok(None)`).
    pub fn get_version_result(&mut self, name: &str) -> Result<Option<Item>, String> {
        let versions = self.get_sdk_versions()?;
        Ok(versions.get(name).cloned())
    }

    /// Latest version (Go `GetLatestVersion`: the first item after sorting).
    pub fn get_latest_version(&mut self) -> Option<(String, Item)> {
        let sorted = self.sorted_versions();
        let first = sorted.into_iter().next()?;
        let item = self.get_version(&first)?;
        Some((first, item))
    }

    pub fn get_installer_config(&mut self) -> Result<InstallerConfig, String> {
        if self.lua.is_none() {
            self.loaded = false;
            self.load()?;
        }
        let lua = self.lua.as_ref().unwrap();
        ic_from_global(lua).ok_or_else(|| "installer config not found".to_string())
    }

    /// Invokes the optional custom handler function (install/postInstall).
    pub fn call_handler(&self, name: &str, args: &[&str]) -> Result<(), String> {
        let lua = self.lua.as_ref().ok_or("plugin not loaded")?;
        let v: mlua::Value = lua
            .globals()
            .get(name)
            .map_err(|_| format!("{name} missing"))?;
        let mlua::Value::Function(f) = v else {
            return Ok(());
        };
        let mut av: Vec<mlua::Value> = Vec::new();
        for a in args {
            av.push(mlua::Value::String(
                lua.create_string(a).map_err(|e| e.to_string())?,
            ));
        }
        let out: mlua::Value = f
            .call(mlua::MultiValue::from_vec(av))
            .map_err(|e| e.to_string())?;
        if crate::req::str_of(&out) != "true" {
            return Err(format!("{name} handler failed"));
        }
        Ok(())
    }
}

/// Whether the plugin directory contains at least one `.lua` file.
fn has_lua_files(dir: &PathBuf) -> bool {
    fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                !e.file_type().map(|t| t.is_dir()).unwrap_or(false) && name.ends_with(".lua")
            })
        })
        .unwrap_or(false)
}

/// Scans the plugin directory and lazily loads metadata (mirrors Go `Plugins`).
pub struct Plugins {
    dir: PathBuf,
}

impl Default for Plugins {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugins {
    /// Returns a plugin directory handle, fetching the plugin bundle on first use when no
    /// `.lua` files are present.
    ///
    /// `paths::plugin_dir()` creates the directory as a side effect, so checking `exists()` (as
    /// the Go code does) can never trigger the download. We instead check for an actual plugin
    /// file, and only attempt the network once per process to avoid repeated stalls/failures.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicBool, Ordering};
        static UPDATE_ATTEMPTED: AtomicBool = AtomicBool::new(false);

        let dir = vmr_core::paths::plugin_dir();
        // Plugins live only in `~/.vmr/plugins` (no compile-time bundle): a missing directory
        // is populated from the upstream `gvcgo/vmr_plugins` archive once per process.
        if !has_lua_files(&dir) && !UPDATE_ATTEMPTED.swap(true, Ordering::Relaxed) {
            let _ = crate::plugins_update::update_plugins();
        }
        Plugins { dir }
    }

    fn lua_files(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                || !name.ends_with(".lua")
                || name.starts_with('_')
            {
                continue;
            }
            out.push(name);
        }
        out.sort();
        out
    }

    /// All plugins (metadata already loaded).
    pub fn load_all(&mut self) -> Vec<Plugin> {
        let mut out = Vec::new();
        for name in self.lua_files() {
            let mut p = Plugin::from_file(name);
            if p.load().is_ok() {
                out.push(p);
            }
        }
        out
    }

    pub fn get_by_plugin_name(&mut self, plugin_name: &str) -> Option<Plugin> {
        self.load_all()
            .into_iter()
            .find(|p| p.plugin_name == plugin_name)
    }

    pub fn get_by_sdk_name(&mut self, sdk_name: &str) -> Option<Plugin> {
        self.load_all().into_iter().find(|p| p.sdk_name == sdk_name)
    }

    pub fn reload(&mut self) {
        let _ = self.load_all();
    }
}

/// For testing and debugging: builds a plugin from a string and runs crawl (for integration tests).
pub fn run_content_crawl(content: &str) -> Result<HashMap<String, Item>, String> {
    let mut p = Plugin::from_content(content.to_string());
    p.load()?;
    p.get_sdk_versions()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_with_plain_lua_plugin() {
        // Minimal plugin: crawl uses the core bindings to produce a version table; verifies load→crawl→platform filtering→ic reading.
        let (os, arch) = os_arch();
        let src = format!(
            r#"
            sdk_name = "testtool"
            plugin_name = "testtool"
            plugin_version = "0.1"
            homepage = "https://example.com"
            ic = vmrNewInstallerConfig()
            ic = vmrAddFlagFiles(ic, "", {{ "bin", "lib" }})
            ic = vmrAddBinaryDirs(ic, "", {{ "bin" }})
            function crawl()
                local vl = vmrNewVersionList()
                local item = {{ url = "https://example.com/x", os = "{os}", arch = "{arch}", installer = vmrInstallerUnarchiver }}
                vmrAddItem(vl, "1.2.3", item)
                local other = {{ url = "u", os = "other", arch = "other", installer = vmrInstallerUnarchiver }}
                vmrAddItem(vl, "9.9.9", other)
                return vl
            end
            "#
        );
        let mut p = Plugin::from_content(src);
        p.load().expect("load 元数据 + ic + crawl 校验");
        assert_eq!(p.plugin_name, "testtool");
        assert_eq!(p.sdk_name, "testtool");
        let ic = p.get_installer_config().expect("ic");
        let fi = ic.flag_files.unwrap();
        assert_eq!(fi.linux, vec!["bin", "lib"]);
        let versions = p.get_sdk_versions().expect("crawl");
        assert_eq!(versions.len(), 1, "仅保留当前平台条目");
        assert_eq!(versions["1.2.3"].os, os);
        // Sorting: only 1.2.3 → the latest is it.
        assert_eq!(p.sorted_versions(), vec!["1.2.3"]);
    }
}
