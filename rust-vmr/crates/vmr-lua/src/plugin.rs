use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lua_engine::LuaEngine;
use crate::types::*;

pub struct Plugin {
    pub meta: PluginMeta,
    pub version_list: HashMap<String, Item>,
    engine: Option<LuaEngine>,
    loaded: bool,
}

impl Plugin {
    /// Create a new Plugin from a .lua file.
    pub fn new(file_path: &std::path::Path) -> mlua::Result<Self> {
        let mut plugin = Plugin {
            meta: PluginMeta::default(),
            version_list: HashMap::new(),
            engine: None,
            loaded: false,
        };
        plugin.meta.file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.lua")
            .to_string();
        plugin.load_lua(file_path)?;
        Ok(plugin)
    }

    fn load_lua(&mut self, file_path: &std::path::Path) -> mlua::Result<()> {
        let engine = LuaEngine::new()?;
        let content = fs::read_to_string(file_path)
            .map_err(|e| mlua::Error::external(format!("Cannot read plugin: {}", e)))?;
        engine.lua.load(&content).exec()?;

        let globals = engine.lua.globals();

        // Extract metadata
        self.meta.plugin_name =
            globals
                .get::<String>(lua_items::PLUGIN_NAME)
                .unwrap_or_default();
        self.meta.sdk_name =
            globals
                .get::<String>(lua_items::SDK_NAME)
                .unwrap_or_default();
        self.meta.plugin_version =
            globals
                .get::<String>(lua_items::PLUGIN_VERSION)
                .unwrap_or_default();
        self.meta.prequisite =
            globals
                .get::<String>(lua_items::PREQUISITE)
                .unwrap_or_default();
        self.meta.homepage =
            globals
                .get::<String>(lua_items::HOMEPAGE)
                .unwrap_or_default();

        self.engine = Some(engine);
        self.loaded = true;
        Ok(())
    }

    pub fn get_sdk_versions(&mut self) -> mlua::Result<&HashMap<String, Item>> {
        if !self.loaded {
            return Ok(&self.version_list);
        }

        // Try cache first
        self.load_from_cache();
        if !self.version_list.is_empty() {
            return Ok(&self.version_list);
        }

        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| mlua::Error::external("Lua engine not initialized"))?;
        let globals = engine.lua.globals();

        // Call crawl() function
        let crawl: mlua::Function = globals
            .get(lua_items::CRAWLER)
            .map_err(|_| mlua::Error::external("crawl function not found"))?;

        // crawl() returns a Lua table: { [version_name] = { item1, item2, ... }, ... }
        let raw_table: mlua::Table = crawl.call(())?;

        let current_os = std::env::consts::OS;
        let current_arch = std::env::consts::ARCH;

        for pair in raw_table.pairs::<String, mlua::Value>() {
            let (vname, items_val) = pair?;
            if let Some(items_table) = items_val.as_table() {
                for i in 1..=items_table.raw_len() {
                    if let Ok(item_table) = items_table.get::<mlua::Table>(i) {
                        let item_os: String = item_table.get("os").unwrap_or_default();
                        let item_arch: String = item_table.get("arch").unwrap_or_default();

                        // Only include items matching current OS/Arch (empty = any)
                        if (!item_os.is_empty() && item_os != current_os)
                            || (!item_arch.is_empty() && item_arch != current_arch)
                        {
                            continue;
                        }

                        let url: String = item_table.get("url").unwrap_or_default();
                        let installer: String =
                            item_table.get("installer").unwrap_or_default();
                        let sum: String = item_table.get("sum").unwrap_or_default();
                        let sum_type: String =
                            item_table.get("sum_type").unwrap_or_default();
                        let size: i64 = item_table.get("size").unwrap_or(0);
                        let lts: String = item_table.get("lts").unwrap_or_default();
                        let extra: String = item_table.get("extra").unwrap_or_default();

                        let version_item = Item {
                            url,
                            arch: item_arch,
                            os: item_os,
                            sum,
                            sum_type,
                            size,
                            installer,
                            lts,
                            extra,
                        };
                        self.version_list.insert(vname.clone(), version_item);
                        break; // Take first matching item for this version
                    }
                }
            }
        }

        self.save_to_cache();
        Ok(&self.version_list)
    }

    fn cache_file_path(&self) -> PathBuf {
        let cache = vmr_config::paths::get_cache_dir();
        cache
            .join(&self.meta.plugin_name)
            .join(format!("{}.versions.json", self.meta.plugin_name))
    }

    fn load_from_cache(&mut self) {
        let path = self.cache_file_path();
        if !path.exists() {
            return;
        }

        // Check file age
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(dur) = modified.duration_since(UNIX_EPOCH) {
                    let age = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64
                        - dur.as_secs() as i64;
                    let retention = vmr_config::conf::get_cache_retention_time();
                    if age > retention {
                        return;
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(vl) = serde_json::from_str::<HashMap<String, Item>>(&content) {
                self.version_list = vl;
            }
        }
    }

    fn save_to_cache(&self) {
        if self.version_list.is_empty() {
            return;
        }
        let path = self.cache_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.version_list) {
            if json.len() > 10 {
                let _ = fs::write(&path, &json);
            }
        }
    }

    pub fn get_latest_version(&mut self) -> Option<(String, Item)> {
        self.get_sdk_versions().ok()?;
        let mut versions: Vec<String> = self.version_list.keys().cloned().collect();
        vmr_utils::version::sort_versions_desc(&mut versions);
        let vname = versions.first()?.clone();
        let item = self.version_list.get(&vname)?.clone();
        Some((vname, item))
    }

    pub fn get_version(&mut self, version_name: &str) -> Option<Item> {
        self.get_sdk_versions().ok()?;
        self.version_list.get(version_name).cloned()
    }
}
