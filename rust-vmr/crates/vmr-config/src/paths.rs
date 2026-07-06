use std::io;
use std::path::PathBuf;

/// Returns the user's home directory or an error
fn home_dir() -> io::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))
}

/// ~/.vmr — the VMR work directory (created if missing)
pub fn get_vmr_work_dir() -> PathBuf {
    let home = home_dir().expect("home directory not found");
    let p = home.join(super::conf::VMR_WORK_DIR_NAME);
    let _ = std::fs::create_dir_all(&p);
    p
}

/// ~/.vmr/conf.toml
pub fn get_vmr_conf_file_path() -> PathBuf {
    get_vmr_work_dir().join("conf.toml")
}

/// Versions directory:
/// Uses VMR_SDK_INSTALLATION_DIR env if set → {env}/versions, else ~/.vmr/versions
pub fn get_versions_dir() -> PathBuf {
    let vp = if let Ok(dir) = std::env::var(super::conf::VMR_SDK_INSTALLATION_DIR) {
        if dir.is_empty() {
            get_vmr_work_dir().join("versions")
        } else {
            PathBuf::from(dir).join("versions")
        }
    } else {
        get_vmr_work_dir().join("versions")
    };
    let _ = std::fs::create_dir_all(&vp);
    vp
}

/// Cache directory: parent of versions_dir + /cache
pub fn get_cache_dir() -> PathBuf {
    let versions_dir = get_versions_dir();
    let parent = versions_dir.parent().unwrap_or(&versions_dir);
    let p = parent.join("cache");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Temp directory for unarchiving SDK files: ~/.vmr/temp
pub fn get_temp_dir() -> PathBuf {
    let p = get_vmr_work_dir().join("temp");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Directory for SDK installation config files: ~/.vmr/install_confs
pub fn get_sdk_installation_conf_dir() -> PathBuf {
    let p = get_vmr_work_dir().join("install_confs");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Plugin directory for Lua plugins: ~/.vmr/plugins
pub fn get_plugin_dir() -> PathBuf {
    let p = get_vmr_work_dir().join("plugins");
    let _ = std::fs::create_dir_all(&p);
    p
}
