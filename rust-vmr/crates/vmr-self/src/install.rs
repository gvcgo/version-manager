use crate::old_versions;
use crate::update;
use crate::uninstall;

/// Install the vmr binary to ~/.vmr/, write shell integration, and generate
/// helper scripts.
pub fn install_self() {
    // 1. Handle old versions (migrate from ~/.vm/ to ~/.vmr/)
    old_versions::detect_and_remove_old_versions();

    // 2. Get current executable path
    let exe_path = std::env::current_exe().expect("cannot get executable path");
    let vmr_work_dir = vmr_config::paths::get_vmr_work_dir();

    // If already installed in vmr work dir, nothing to do
    if exe_path.starts_with(&vmr_work_dir) {
        return;
    }

    // 3. Copy executable to ~/.vmr/
    let bin_name = exe_path.file_name().unwrap();
    let install_path = vmr_work_dir.join(bin_name);
    let _ = std::fs::remove_file(&install_path);
    vmr_utils::fs::copy_file(&exe_path, &install_path).expect("copy vmr failed");

    // 4. Write shell integration
    let shell = vmr_shell::unix::new_shell();
    shell.write_vm_env_to_shell();

    // 5. Generate update script
    update::set_update_script();

    // 6. Generate uninstall script
    uninstall::set_uninstall_script();

    // 7. Add custom source alias (unix only)
    add_customed_source_cmd();
}

/// Append a `svmr` alias to the shell config file so that users can
/// re-source the VM env after toggling VM_DISABLE.
fn add_customed_source_cmd() {
    #[cfg(not(windows))]
    {
        let shell = vmr_shell::unix::new_shell();
        let profile_path = shell.conf_path();
        let source_cmd = format!(
            "alias svmr=\"export VM_DISABLE='' && source {}\"",
            profile_path.display()
        );

        match std::fs::read_to_string(&profile_path) {
            Ok(old_content) if !old_content.contains(&source_cmd) => {
                let new_content = format!("{}\n{}", old_content.trim_end(), source_cmd);
                let _ = std::fs::write(&profile_path, new_content);
            }
            _ => {}
        }
    }
}
