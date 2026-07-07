use vmr_self::{install, update, uninstall, old_versions};
use vmr_config::paths;

#[test]
fn test_install_self_already_installed() {
    // This should be a no-op when VMR is not running from ~/.vmr/
    // Just verify it doesn't panic
    install::install_self();
}

#[test]
fn test_remove_current_version_no_panic() {
    // Should not panic even if nothing is installed
    old_versions::remove_current_version();
}

#[test]
fn test_update_script_written() {
    // Call set_update_script and verify files exist
    update::set_update_script();
}

#[test]
fn test_uninstall_script_written() {
    uninstall::set_uninstall_script();
}

#[test]
fn test_update_script_path_exists() {
    update::set_update_script();
    let work_dir = paths::get_vmr_work_dir();
    #[cfg(not(windows))]
    let script = work_dir.join("vmr-update");
    #[cfg(windows)]
    let script = work_dir.join("vmr-update.bat");
    // After calling set_update_script, the file should exist
    assert!(script.exists(), "update script should exist at {:?}", script);
}

#[test]
fn test_uninstall_script_path_exists() {
    uninstall::set_uninstall_script();
    let work_dir = paths::get_vmr_work_dir();
    #[cfg(not(windows))]
    let script = work_dir.join("vmr-uninstall");
    #[cfg(windows)]
    let script = work_dir.join("vmr-uninstall.bat");
    assert!(script.exists(), "uninstall script should exist at {:?}", script);
}
