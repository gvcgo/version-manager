use vmr_install::common;

#[test]
fn test_get_sdk_version_dir() {
    let dir = common::get_sdk_version_dir("test_sdk");
    assert!(dir.exists());
    assert!(dir.to_str().unwrap().contains("test_sdk_versions"));
}

#[test]
fn test_is_sdk_installed_not_installed() {
    // A random SDK name should not be installed
    let installed = common::is_sdk_installed_by_vmr("nonexistent_sdk_test");
    assert!(!installed);
}

#[test]
fn test_file_items_for_current_os() {
    let fi = common::FileItems {
        windows: vec!["win_flag".into()],
        linux: vec!["linux_flag".into()],
        macos: vec!["mac_flag".into()],
    };
    let items = fi.for_current_os();
    assert!(!items.is_empty());
}

#[test]
fn test_installer_config_default() {
    let config = common::InstallerConfig::default();
    assert!(config.flag_files.is_none());
    assert!(!config.flag_dir_excepted);
}
