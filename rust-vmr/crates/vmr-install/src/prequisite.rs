use std::process::Command;

/// Check if miniconda is installed (by looking for conda command)
pub fn is_miniconda_installed() -> bool {
    Command::new("conda")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if coursier is installed (by looking for cs command)
pub fn is_coursier_installed() -> bool {
    let cs_cmd = std::env::var("VMR_COURSIER_PATH").unwrap_or_else(|_| "cs".to_string());
    Command::new(&cs_cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Placeholder — auto-installs miniconda when needed.
/// The actual install logic is in the installer crate's ExeInstaller.
pub fn check_and_install_miniconda() {
    if !is_miniconda_installed() {
        eprintln!("[vmr] Miniconda is not installed. Please install it first.");
        eprintln!("[vmr] Run: vmr use miniconda@latest");
    }
}

/// Placeholder — auto-installs coursier when needed.
pub fn check_and_install_coursier() {
    if !is_coursier_installed() {
        eprintln!("[vmr] Coursier is not installed. Please install it first.");
        eprintln!("[vmr] Run: vmr use coursier@latest");
    }
}
