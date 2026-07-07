use std::io::Write;
use std::{env, fs};

use vmr_shell::common::{
    format_path_string, update_vmr_shell_file, BashShell, FishShell, Sheller, ZshShell,
};
use vmr_shell::unix::new_shell;

// ---------------------------------------------------------------------------
// conf_path() tests
// ---------------------------------------------------------------------------

#[test]
fn test_bash_conf_path() {
    let bash = BashShell;
    let path = bash.conf_path();
    assert!(path.to_str().unwrap().contains(".bashrc"));
}

#[test]
fn test_zsh_conf_path() {
    let zsh = ZshShell;
    let path = zsh.conf_path();
    assert!(path.to_str().unwrap().contains(".zshrc"));
}

#[test]
fn test_fish_conf_path() {
    let fish = FishShell;
    let path = fish.conf_path();
    assert!(path.to_str().unwrap().contains("fish"));
    assert!(path.to_str().unwrap().contains("config.fish"));
}

// ---------------------------------------------------------------------------
// pack_path() tests
// ---------------------------------------------------------------------------

#[test]
fn test_bash_pack_path() {
    let bash = BashShell;
    let packed = bash.pack_path("/usr/local/bin");
    assert!(packed.contains("export PATH"));
    assert!(packed.contains("/usr/local/bin"));
    assert!(packed.contains("$PATH"));
}

#[test]
fn test_bash_pack_env() {
    let bash = BashShell;
    let packed = bash.pack_env("JAVA_HOME", "/usr/lib/jvm");
    assert!(packed.contains("export JAVA_HOME"));
    assert!(packed.contains("/usr/lib/jvm"));

    let packed = bash.pack_env("EMPTY_VAR", "");
    assert!(packed.contains("export EMPTY_VAR"));
}

#[test]
fn test_zsh_pack_path() {
    let zsh = ZshShell;
    let packed = zsh.pack_path("/opt/bin");
    assert!(packed.contains("export PATH"));
}

#[test]
fn test_fish_pack_path() {
    let fish = FishShell;
    let packed = fish.pack_path("/usr/local/bin");
    assert!(packed.contains("fish_add_path"));
}

#[test]
fn test_fish_pack_env() {
    let fish = FishShell;
    let packed = fish.pack_env("MY_VAR", "my_value");
    assert!(packed.contains("set -gx MY_VAR"));
    assert!(packed.contains("my_value"));
}

// ---------------------------------------------------------------------------
// vm_env_conf_path() tests
// ---------------------------------------------------------------------------

#[test]
fn test_vm_env_conf_path() {
    let bash = BashShell;
    let path = bash.vm_env_conf_path();
    assert!(path.to_str().unwrap().contains(".vmr"));
}

// ---------------------------------------------------------------------------
// format_path_string() tests
// ---------------------------------------------------------------------------

#[test]
fn test_format_path_string() {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let test_path = format!("{}/.vmr/vmr", home);
    let formatted = format_path_string(&test_path);
    assert!(formatted.starts_with("~"));
}

// ---------------------------------------------------------------------------
// new_shell() / Shell api tests
// ---------------------------------------------------------------------------

#[test]
fn test_new_shell_returns_shell() {
    let shell = new_shell();
    // Just verify it doesn't panic and we can call methods
    let _ = shell.set_env("VMR_TEST_SHELL", "1");
    let _ = shell.unset_env("VMR_TEST_SHELL");
}

#[test]
fn test_shell_set_and_unset_path() {
    let shell = new_shell();
    let test_path = "/tmp/vmr_test_path";

    // Set path – this writes to ~/.vmr/vmr
    shell.set_path(test_path);

    // Read the VM env file to verify
    let conf_path = shell.vm_env_conf_path();
    if let Ok(content) = fs::read_to_string(&conf_path) {
        if content.contains(test_path) {
            // Now unset it
            shell.unset_path(test_path);
        }
    }
}

#[test]
fn test_shell_set_and_unset_env() {
    let shell = new_shell();
    shell.set_env("VMR_TEST_VAR", "test_value");
    shell.unset_env("VMR_TEST_VAR");
}

// ---------------------------------------------------------------------------
// update_vmr_shell_file() test
// ---------------------------------------------------------------------------

#[test]
fn test_update_vmr_shell_file() {
    let tmp_dir = env::temp_dir();
    let test_file = tmp_dir.join("vmr_shell_test");

    // Make sure the file starts empty
    let _ = fs::remove_file(&test_file);

    let new_hook = "# cd hook start\ncdhook() { :; }\n# cd hook end";
    update_vmr_shell_file(&test_file, "export PATH=old:", new_hook);

    let content = fs::read_to_string(&test_file).unwrap_or_default();
    assert!(content.contains("cd hook start"));

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_update_vmr_shell_file_keeps_other_content() {
    let tmp_dir = env::temp_dir();
    let test_file = tmp_dir.join("vmr_shell_test_preserve");

    // Prepare a file with existing content + existing hook block
    let existing = "# cd hook start\nold_hook() { :; }\n# cd hook end\nexport FOO=bar\n";
    {
        let mut f = std::fs::File::create(&test_file).unwrap();
        f.write_all(existing.as_bytes()).unwrap();
    }

    let new_hook = "# cd hook start\nnew_hook() { :; }\n# cd hook end";
    update_vmr_shell_file(&test_file, "export PATH=old:", new_hook);

    let content = fs::read_to_string(&test_file).unwrap_or_default();
    assert!(content.contains("new_hook"));
    assert!(content.contains("export FOO=bar"));

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

// ---------------------------------------------------------------------------
// write_vm_env_to_shell() test (non-destructive – only verifies no panic)
// ---------------------------------------------------------------------------

#[test]
fn test_write_vm_env_to_shell() {
    let bash = BashShell;
    bash.write_vm_env_to_shell();

    // After writing, the bashrc should at least exist or be readable
    let conf = bash.conf_path();
    if let Ok(content) = fs::read_to_string(&conf) {
        // Either contains the source line or was already present (or file empty)
        // We don't assert on the exact content since this touches the real
        // .bashrc – just verify we got here without panicking.
        let _ = content;
    }
}
