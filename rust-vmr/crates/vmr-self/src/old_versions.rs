use std::fs;

/// Remove the current VMR version, cache, shell RC entries, and the
/// ~/.vmr/ work directory.  Used by the uninstall-self command.
pub fn remove_current_version() {
    let version_dir = vmr_config::paths::get_versions_dir();
    let cache_dir = vmr_config::paths::get_cache_dir();
    let _ = std::fs::remove_dir_all(&version_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);

    // Clean shell RC files (remove the guarded source block)
    #[cfg(not(windows))]
    {
        let shell = vmr_shell::unix::new_shell();
        let conf_path = shell.conf_path();
        if let Ok(content) = fs::read_to_string(&conf_path) {
            let new_content = content
                .replace(
                    concat!(
                        "# vm_envs start\n",
                        "if [ -z \"${VM_DISABLE}\" ]; then\n",
                        "    . ~/.vmr/vmr\n",
                        "fi\n",
                        "# vm_envs end",
                    ),
                    "",
                )
                .replace(
                    concat!(
                        "# vm_envs start\n",
                        "if not test \"${VM_DISABLE}\"\n",
                        "    . ~/.vmr/vmr.fish\n",
                        "end\n",
                        "# vm_envs end",
                    ),
                    "",
                );
            let _ = fs::write(&conf_path, new_content.trim());
        }
    }

    let _ = std::fs::remove_dir_all(vmr_config::paths::get_vmr_work_dir());
}

/// Detect old ~/.vm/ installations and remove them, printing a warning.
pub fn detect_and_remove_old_versions() {
    // Home directory derived from the vmr work dir (avoids adding dirs dep)
    let work_dir = vmr_config::paths::get_vmr_work_dir(); // ~/.vmr
    let home = match work_dir.parent() {
        Some(p) => p,
        None => return,
    };
    let old_work_dir = home.join(".vm");
    let old_binary = if cfg!(windows) {
        old_work_dir.join("vmr.exe")
    } else {
        old_work_dir.join("vmr")
    };

    if !old_binary.exists() {
        return;
    }

    // Old version detected — print warning and remove
    eprintln!(
        "[vmr] An old version of VMR ({}) is detected. Removing...",
        old_work_dir.display()
    );
    let _ = std::fs::remove_dir_all(&old_work_dir);
}
