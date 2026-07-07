use std::fs;
use vmr_config::paths;

const UNINSTALL_SCRIPT_NAME: &str = "vmr-uninstall";

/// Write the vmr-uninstall helper script (~/.vmr/vmr-uninstall[.bat|.sh]).
pub fn set_uninstall_script() {
    let work_dir = paths::get_vmr_work_dir();

    #[cfg(windows)]
    {
        let script = format!(
            "cd %HOMEPATH%\nvmr Uins\nrmdir /s /q {}",
            work_dir.display()
        );
        let bat_path = work_dir.join(format!("{}.bat", UNINSTALL_SCRIPT_NAME));
        fs::write(&bat_path, &script).ok();
    }

    #[cfg(not(windows))]
    {
        let script = format!(
            "#!/bin/sh\ncd ~\nvmr Uins\nrm -rf {}",
            work_dir.display()
        );
        let script_path = work_dir.join(UNINSTALL_SCRIPT_NAME);
        fs::write(&script_path, &script).ok();

        // Make it executable
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755));
    }
}
