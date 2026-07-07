use std::fs;

use crate::plugin::Plugin;

pub struct Plugins {
    plugins: std::collections::HashMap<String, Plugin>,
}

impl Plugins {
    pub fn new() -> Self {
        Plugins {
            plugins: std::collections::HashMap::new(),
        }
    }

    /// Load all .lua plugins from the plugin directory
    pub fn load_all(&mut self) {
        if !self.plugins.is_empty() {
            return;
        }

        let plugin_dir = vmr_config::paths::get_plugin_dir();
        if let Ok(entries) = fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "lua") {
                    if let Ok(pl) = Plugin::new(&path) {
                        let name = pl.meta.plugin_name.clone();
                        self.plugins.insert(name, pl);
                    }
                }
            }
        }
    }

    pub fn get_plugin(&mut self, plugin_name: &str) -> Option<&mut Plugin> {
        self.load_all();
        self.plugins.get_mut(plugin_name)
    }

    pub fn get_plugin_by_sdk(&mut self, sdk_name: &str) -> Option<&mut Plugin> {
        self.load_all();
        self.plugins
            .values_mut()
            .find(|p| p.meta.sdk_name == sdk_name)
    }

    pub fn get_plugin_names(&mut self) -> Vec<String> {
        self.load_all();
        self.plugins.keys().cloned().collect()
    }
}
