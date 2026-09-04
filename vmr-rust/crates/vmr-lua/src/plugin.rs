//! 插件生命周期（对齐 Go `internal/luapi/plugin/{plugin.go,plugins.go,fromlua.go}`）。
//!
//! 语义要点：
//! - 元数据（sdk_name/plugin_name/plugin_version/homepage/prequisite）读取：
//!   全局字符串；若为函数则调用取返回值（Go `GetLuaConfItemString`）。
//! - Load 校验：plugin_name/sdk_name/homepage/ic/crawl 必须存在。
//! - 版本获取：缓存优先（`<cache>/<plugin>/<plugin>.versions.json`，retention
//!   内新鲜且 cache 未禁用），否则实时执行 `crawl()` 并按当前 os/arch 过滤
//!   （保留**最后一个**匹配 Item，对齐 Go），写缓存。
//! - 安装回调：`install`/`postInstall` 全局函数存在时以参数调用，返回串须为
//!   "true" 才算成功（Go `getFuncFromLua` 语义）。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use mlua::Lua;

use crate::bindings::new_runtime;
use crate::installer_conf::ic_from_global;
use crate::types::{InstallerConfig, Item};
use crate::utils_bind::os_arch;
use crate::version::{VersionListUD, filter_current_platform};

pub const PLUGIN_NAME: &str = "plugin_name";
pub const SDK_NAME: &str = "sdk_name";
pub const PLUGIN_VERSION: &str = "plugin_version";
pub const PREQUISITE: &str = "prequisite";
pub const HOMEPAGE: &str = "homepage";
pub const CRAWL: &str = "crawl";
pub const POST_INSTALL: &str = "postInstall";
pub const CUSTOM_INSTALL: &str = "install";

/// 单个插件。
pub struct Plugin {
    file_name: Option<String>,
    file_content: Option<String>,
    lua: Option<Lua>,
    pub plugin_name: String,
    pub sdk_name: String,
    pub plugin_version: String,
    pub prequisite: String,
    pub homepage: String,
    /// 当前平台版本表：版本 → Item。
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

    /// 加载脚本并解析元数据（对齐 Go `Load`）。
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

    /// 当前平台版本表（缓存优先；否则实时 crawl + 过滤 + 缓存）。
    pub fn get_sdk_versions(&mut self) -> Result<HashMap<String, Item>, String> {
        if self.lua.is_none() {
            self.loaded = false;
            self.load()?;
        }
        self.load_cache();
        if !self.versions.is_empty() {
            return Ok(self.versions.clone());
        }
        let lua = self.lua.as_ref().unwrap();
        let crawl = lua.globals().get::<mlua::Function>(CRAWL).map_err(|_| {
            format!(
                "invalid plugin: missing crawl function: {}",
                self.plugin_name
            )
        })?;
        let result: mlua::Value = crawl
            .call(())
            .map_err(|e| format!("crawl failed for {}: {e}", self.plugin_name))?;
        if let mlua::Value::UserData(ud) = result {
            if let Ok(vl) = ud.borrow::<VersionListUD>() {
                let (os, arch) = os_arch();
                self.versions = filter_current_platform(&vl.0, &os, &arch);
            }
        }
        self.save_cache();
        Ok(self.versions.clone())
    }

    /// 版本名降序（vmr 语义）。
    pub fn sorted_versions(&mut self) -> Vec<String> {
        if self.versions.is_empty() {
            let _ = self.get_sdk_versions();
        }
        let mut names: Vec<String> = self.versions.keys().cloned().collect();
        vmr_utils::semver::sort_versions(&mut names);
        names
    }

    pub fn get_version(&mut self, name: &str) -> Option<Item> {
        if self.versions.is_empty() {
            let _ = self.get_sdk_versions();
        }
        self.versions.get(name).cloned()
    }

    /// 最新版本（Go `GetLatestVersion`：排序后首项）。
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

    /// 调可选自定义处理函数（install/postInstall）。
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

/// 扫描插件目录并懒加载元数据（对齐 Go `Plugins`）。
pub struct Plugins {
    dir: PathBuf,
}

impl Default for Plugins {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugins {
    /// 目录不存在时先自动更新（对齐 Go NewPlugins）。
    pub fn new() -> Self {
        let dir = vmr_core::paths::plugin_dir();
        if !dir.exists() {
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
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) || !name.ends_with(".lua") {
                continue;
            }
            out.push(name);
        }
        out.sort();
        out
    }

    /// 全部插件（已 load 元数据）。
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

/// 测试与调试用：从字符串建插件并执行 crawl（供集成测试）。
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
        // 最小插件：crawl 用核心绑定产出版本表；验证 load→crawl→平台过滤→ic 读取。
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
        // 排序：只有 1.2.3 → 最新即它。
        assert_eq!(p.sorted_versions(), vec!["1.2.3"]);
    }
}
