//! Windows PowerShell shell integration.
//!
//! On Windows, VMR manages PATH through user environment variables
//! and writes a PowerShell profile script with a cd-hook.

use std::fs;
use std::path::PathBuf;

/// PowerShell cd-hook script (ported from Go's PowershellHook constant)
pub const POWERSHELL_HOOK: &str = r##"# cd hook start
function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        chdir $args[0]
        vmr use -E
    }
}

function vmrsource {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

Set-Alias -Name cd -Option AllScope -Value cdhook
Set-Alias -Name source -Value vmrsource

if ( "" -eq "$env:VMR_CD_INIT" )
{
    $env:VMR_CD_INIT="vmr_cd_init"
    cd "$(-split $(pwd))"
}
# cd hook end"##;

/// Returns the PowerShell profile path.
pub fn powershell_profile_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
    let ps_dir = home.join("Documents").join("WindowsPowerShell");
    let _ = fs::create_dir_all(&ps_dir);
    ps_dir.join("profile.ps1")
}

/// Write the PowerShell cd-hook into the PowerShell profile.
pub fn write_powershell_hook() {
    let profile_path = powershell_profile_path();

    let content = fs::read_to_string(&profile_path).unwrap_or_default();
    let old_pws_hook = r##"function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        chdir $args[0]
        vmr use -E
    }
}

function vmrsource {
	$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

Set-Alias -Name cd -Option AllScope -Value cdhook
Set-Alias -Name source -Value vmrsource"##;

    let new_content = if content.contains("# cd hook start") {
        // Replace existing hook block between markers
        if let (Some(start), Some(end)) =
            (content.find("# cd hook start"), content.rfind("# cd hook end"))
        {
            let end = end + "# cd hook end".len();
            let before = &content[..start];
            let after = &content[end..];
            format!("{}{}{}", before, POWERSHELL_HOOK, after)
        } else {
            format!("{}\n{}", POWERSHELL_HOOK, content)
        }
    } else if content.contains("cdhook") {
        // Replace old hook (no markers)
        content.replace(&old_pws_hook, POWERSHELL_HOOK)
    } else if content.trim().is_empty() {
        POWERSHELL_HOOK.to_string()
    } else {
        format!("{}\n{}", POWERSHELL_HOOK, content.trim())
    };

    let _ = fs::write(&profile_path, new_content.trim());
}

/// Remove the PowerShell cd-hook from the profile.
pub fn remove_powershell_hook() {
    let profile_path = powershell_profile_path();
    if let Ok(content) = fs::read_to_string(&profile_path) {
        let new_content = if let (Some(start), Some(end)) =
            (content.find("# cd hook start"), content.rfind("# cd hook end"))
        {
            let end = end + "# cd hook end".len();
            let before = &content[..start];
            let after = &content[end..];
            format!("{}{}", before, after)
        } else {
            content
        };
        let new_content = new_content.trim().replace("\n\n\n", "\n\n");
        let _ = fs::write(&profile_path, new_content.trim());
    }
}

// ---------------------------------------------------------------------------
// MinGW Bash compat (ported from win_patch.go)
// ---------------------------------------------------------------------------

/// VMR cd hook snippet for MinGW bash on Windows.
pub const MINGW_BASH_CD_HOOK_TEMPLATE: &str = r##"# cd hook start
export PATH="%s:${PATH}"

if ! alias | grep -q cdhook; then
	cdhook() {
		if [ $# -eq 0 ]; then
			cd || true
		else
			cd "$@" && vmr use -E
		fi
	}
	alias cd='cdhook'
fi

if [ -z "${VMR_CD_INIT:-}" ]; then
        VMR_CD_INIT="vmr_cd_init"
        cd "$(pwd)" || true
fi
# cd hook end"##;

/// Returns the MinGW bash profile path (~/.bashrc).
pub fn mingw_bash_profile_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\"))
        .join(".bashrc")
}

/// Write cd hook for MinGW bash on Windows.
pub fn write_mingw_bash_cd_hook(vmr_install_dir: &str) {
    let profile = mingw_bash_profile_path();
    let content = fs::read_to_string(&profile).unwrap_or_default();
    let cd_hook = format!(MINGW_BASH_CD_HOOK_TEMPLATE, vmr_install_dir);

    if content.contains(&cd_hook) {
        return;
    }

    let new_content = if content.trim().is_empty() {
        cd_hook
    } else {
        format!("{}\n{}", content.trim_end(), cd_hook)
    };
    let _ = fs::write(&profile, new_content);
}

/// Add a PATH export line to MinGW bash profile.
pub fn add_mingw_path_export(path: &str) {
    let profile = mingw_bash_profile_path();
    let content = fs::read_to_string(&profile).unwrap_or_default();
    let export_line = format!("export PATH=\"${{PATH}}:{}\"", path);

    if content.contains(&export_line) {
        return;
    }

    let new_content = if content.trim().is_empty() {
        export_line
    } else {
        format!("{}\n{}", content.trim_end(), export_line)
    };
    let _ = fs::write(&profile, new_content);
}
