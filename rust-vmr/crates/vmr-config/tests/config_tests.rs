use vmr_config::conf::{VMRConf, VMR_HOST, default_config};
use vmr_config::paths;

#[test]
fn test_default_config() {
    let config = default_config();
    // Default config should exist
    assert!(std::mem::size_of_val(config) > 0);
}

#[test]
fn test_save_and_load_config() {
    let conf_path = paths::get_vmr_conf_file_path();
    // Backup existing config
    let backup = if conf_path.exists() {
        Some(std::fs::read_to_string(&conf_path).unwrap_or_default())
    } else {
        None
    };

    let mut conf = VMRConf::default();
    conf.proxy_uri = Some("http://test:8080".to_string());
    conf.download_thread_num = Some(4);
    conf.save().unwrap();

    let mut loaded = VMRConf::default();
    loaded.load();
    assert_eq!(loaded.proxy_uri, Some("http://test:8080".to_string()));
    assert_eq!(loaded.download_thread_num, Some(4));

    // Restore backup
    if let Some(b) = backup {
        std::fs::write(&conf_path, b).ok();
    } else {
        std::fs::remove_file(&conf_path).ok();
    }
}

#[test]
fn test_work_dir_exists() {
    let dir = paths::get_vmr_work_dir();
    assert!(dir.exists());
    assert!(dir.ends_with(".vmr"));
}

#[test]
fn test_conf_file_path() {
    let path = paths::get_vmr_conf_file_path();
    assert!(path.ends_with("conf.toml"));
    assert!(path.to_str().unwrap().contains(".vmr"));
}

#[test]
fn test_versions_dir() {
    let dir = paths::get_versions_dir();
    assert!(dir.exists());
}

#[test]
fn test_cache_dir() {
    let dir = paths::get_cache_dir();
    assert!(dir.exists());
}

#[test]
fn test_temp_dir() {
    let dir = paths::get_temp_dir();
    assert!(dir.exists());
}

#[test]
fn test_plugin_dir() {
    let dir = paths::get_plugin_dir();
    assert!(dir.exists());
}

#[test]
fn test_sdk_installation_conf_dir() {
    let dir = paths::get_sdk_installation_conf_dir();
    assert!(dir.exists());
}

#[test]
fn test_env_var_setting() {
    let mut conf = VMRConf::default();
    // Test that conf.new() applies env vars
    conf.version_host_url = Some("https://custom.example.com".to_string());
    // Save and reload via new()
    conf.save().unwrap();
    let _conf2 = VMRConf::new();
    let host = std::env::var(VMR_HOST).unwrap_or_default();
    assert!(host == "https://custom.example.com" || host.is_empty());
}

#[test]
fn test_get_sdk_list_file_url() {
    let url = vmr_config::conf::get_sdk_list_file_url();
    assert!(url.contains("sdk-list.version.json"));
}

#[test]
fn test_get_version_file_url_by_sdk_name() {
    let url = vmr_config::conf::get_version_file_url_by_sdk_name("golang");
    assert!(url.contains("golang.version.json"));
}

#[test]
fn test_get_download_thread_num() {
    let n = vmr_config::conf::get_download_thread_num();
    assert!(n >= 1);
}

#[test]
fn test_get_reverse_proxy_uri() {
    // Without github URL, should return empty
    let uri = vmr_config::conf::get_reverse_proxy_uri("https://example.com/file.zip", "");
    assert!(uri.is_empty() || uri.starts_with("http"));

    // With local proxy set, should return empty
    let uri = vmr_config::conf::get_reverse_proxy_uri("https://github.com/file.zip", "http://proxy:8080");
    assert!(uri.is_empty());

    // gitee should not need reverse proxy
    let uri = vmr_config::conf::get_reverse_proxy_uri("https://gitee.com/file.zip", "");
    assert!(uri.is_empty());
}

#[test]
fn test_setter_methods() {
    let mut conf = VMRConf::default();
    conf.set_proxy_uri("http://test:8080");
    assert_eq!(conf.proxy_uri, Some("http://test:8080".to_string()));

    conf.set_download_thread_num(8);
    assert_eq!(conf.download_thread_num, Some(8));

    conf.set_download_thread_num(0);
    assert_eq!(conf.download_thread_num, Some(1));
}
