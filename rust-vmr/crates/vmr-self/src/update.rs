use std::fs;
use vmr_config::conf::DEFAULT_DOMAIN;

const UPDATE_SCRIPT_NAME: &str = "vmr-update";

/// Write the vmr-update helper script (~/.vmr/vmr-update[.bat|.sh]).
pub fn set_update_script() {
    let work_dir = vmr_config::paths::get_vmr_work_dir();

    #[cfg(windows)]
    {
        let win_script = format!(
            "cd %HOMEPATH%\npowershell -c \"irm https://scripts.{}/windows | iex\"",
            DEFAULT_DOMAIN
        );
        let bat_path = work_dir.join(format!("{}.bat", UPDATE_SCRIPT_NAME));
        fs::write(&bat_path, &win_script).ok();

        let mingw_script = format!("#!/bin/sh\ncd ~\npowershell {}", bat_path.display());
        let mingw_path = work_dir.join(format!("{}.sh", UPDATE_SCRIPT_NAME));
        fs::write(&mingw_path, &mingw_script).ok();
    }

    #[cfg(not(windows))]
    {
        let unix_script = format!(
            "#!/bin/sh\ncd ~\ncurl --proto '=https' --tlsv1.2 -sSf https://scripts.{} | sh",
            DEFAULT_DOMAIN
        );
        let script_path = work_dir.join(UPDATE_SCRIPT_NAME);
        fs::write(&script_path, &unix_script).ok();
    }
}
