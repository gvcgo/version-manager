use std::fs;

const PLUGINS_DOWNLOAD_URL: &str =
    "https://github.com/gvcgo/vmr_plugins/archive/refs/heads/main.zip";

/// Download and extract the latest plugins from GitHub.
pub fn update_plugins() -> Result<(), String> {
    let temp_dir = vmr_config::paths::get_temp_dir();
    let plugin_dir = vmr_config::paths::get_plugin_dir();
    let _ = fs::create_dir_all(&plugin_dir);

    // Download plugins zip
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let zip_name = "vmr_plugins.zip";
    let zip_path = temp_dir.join(zip_name);

    let mut response = client
        .get(PLUGINS_DOWNLOAD_URL)
        .send()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let mut file =
        fs::File::create(&zip_path).map_err(|e| format!("Cannot create file: {}", e))?;
    std::io::copy(&mut response, &mut file).map_err(|e| format!("Write failed: {}", e))?;
    drop(file);

    // Extract
    vmr_utils::archive::extract(&zip_path, &temp_dir)
        .map_err(|e| format!("Extract failed: {}", e))?;

    // Find and copy .lua files from the extracted directory
    let mut finder = vmr_utils::fs::HomeDirFinder::new(vec!["go.lua".into()]);
    finder.find(&temp_dir);

    if let Some(src_dir) = finder.get_dir_name() {
        if let Ok(entries) = fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "lua") {
                    let dest = plugin_dir.join(
                        path.file_name()
                            .expect("valid file name in extracted plugins"),
                    );
                    let _ = fs::remove_file(&dest);
                    let _ = fs::copy(&path, &dest);
                }
            }
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}
