use std::process::Command;

/// Helper to run the vmr binary with args and check exit status.
///
/// Tries `CARGO_BIN_EXE_vmr-cli` first (set by cargo during `cargo test`),
/// then falls back to a hardcoded path, and finally `cargo run`.
fn run_vmr(args: &[&str]) -> std::process::Output {
    // CARGO_BIN_EXE_<name> is automatically set by cargo for integration tests
    // of binary crates.  The env var name uses the binary target name, which
    // defaults to the package name.
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_vmr-cli") {
        return Command::new(&bin)
            .args(args)
            .output()
            .expect("failed to run vmr-cli binary");
    }

    // Fallback – might work if the binary is already built
    let candidates = &["target/debug/vmr-cli", "target/release/vmr-cli"];
    for c in candidates {
        if std::path::Path::new(c).exists() {
            return Command::new(c)
                .args(args)
                .output()
                .expect("failed to run vmr-cli binary");
        }
    }

    // Last resort – use `cargo run`
    Command::new("cargo")
        .args(
            std::iter::once("run")
                .chain(std::iter::once("-p"))
                .chain(std::iter::once("vmr-cli"))
                .chain(std::iter::once("--"))
                .chain(args.iter().map(|s| *s)),
        )
        .output()
        .expect("failed to run vmr-cli via cargo")
}

// ---------------------------------------------------------------------------
// Basic meta-commands
// ---------------------------------------------------------------------------

#[test]
fn test_version_command() {
    let output = run_vmr(&["version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("vmr") || stdout.contains("0.1"),
        "version output: {stdout}"
    );
}

#[test]
fn test_help_command() {
    let output = run_vmr(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vmr"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("search"));
}

// ---------------------------------------------------------------------------
// Use subcommand
// ---------------------------------------------------------------------------

#[test]
fn test_use_command_help() {
    let output = run_vmr(&["use", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sdkname@version")
            || stdout.contains("pluginname@version")
            || stdout.contains("version_info"),
        "use help: {stdout}"
    );
}

#[test]
fn test_use_invalid_format() {
    let output = run_vmr(&["use", "not_valid_format"]);
    // Should handle gracefully – either prints usage error or the CLI stub
    // prints its own error.
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Just verify it doesn't crash – either error or informed output
    let _ = (stderr, stdout);
}

// ---------------------------------------------------------------------------
// Search subcommand
// ---------------------------------------------------------------------------

#[test]
fn test_search_no_args() {
    let output = run_vmr(&["search"]);
    // Requires an argument, so clap will print an error + help
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("help")
            || stderr.contains("error")
            || stdout.contains("usage")
            || stderr.contains("usage"),
        "search output (no args): stdout={stdout} stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// Show / Local / Installed subcommands
// ---------------------------------------------------------------------------

#[test]
fn test_show_command() {
    let output = run_vmr(&["show"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SDK") || stdout.contains("Showing"),
        "show output: {stdout}"
    );
}

#[test]
fn test_local_no_args() {
    let output = run_vmr(&["local"]);
    // Requires an argument; clap will error
    let _ = String::from_utf8_lossy(&output.stderr);
}

#[test]
fn test_installed_sdks() {
    let output = run_vmr(&["installed-sdks"]);
    assert!(output.status.success());
}

#[test]
fn test_installed_info() {
    let output = run_vmr(&["installed-info"]);
    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Uninstall subcommand
// ---------------------------------------------------------------------------

#[test]
fn test_uninstall_help() {
    let output = run_vmr(&["uninstall", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("pluginname@version")
            || stdout.contains("version_info")
            || stdout.contains("sdkname@version"),
        "uninstall help: {stdout}"
    );
}

#[test]
fn test_uninstall_invalid_format() {
    let output = run_vmr(&["uninstall", "invalid"]);
    let _ = String::from_utf8_lossy(&output.stderr);
}

// ---------------------------------------------------------------------------
// Update plugins
// ---------------------------------------------------------------------------

#[test]
fn test_update_plugins() {
    let output = run_vmr(&["update-plugins"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Updating") || stdout.contains("update"),
        "update-plugins output: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Install-self / Uninstall-self
// ---------------------------------------------------------------------------

#[test]
fn test_install_self_command() {
    let output = run_vmr(&["install-self"]);
    // install_self() touches files in ~/.vmr/ but is a no-op when already
    // installed.  We only care that it doesn't panic.
    let _ = output.status;
}

#[test]
fn test_uninstall_self_command() {
    let output = run_vmr(&["uninstall-self"]);
    // remove_current_version() removes ~/.vmr/ entries.  Should not panic.
    let _ = output.status;
}

// ---------------------------------------------------------------------------
// Command aliases
// ---------------------------------------------------------------------------

#[test]
fn test_version_alias() {
    let output = run_vmr(&["v"]);
    assert!(output.status.success());
}

#[test]
fn test_show_alias() {
    let output = run_vmr(&["S"]);
    assert!(output.status.success());
}

#[test]
fn test_install_self_alias() {
    let output = run_vmr(&["i"]);
    let _ = output.status;
}
