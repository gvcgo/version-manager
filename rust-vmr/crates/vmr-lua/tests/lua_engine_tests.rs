use vmr_lua::lua_engine::LuaEngine;

// ---------------------------------------------------------------------------
// OS/Arch
// ---------------------------------------------------------------------------

#[test]
fn test_get_os_arch() {
    let engine = LuaEngine::new().unwrap();
    let result: (String, String) = engine
        .lua
        .load(
            r#"
        os, arch = vmrGetOsArch()
        return os, arch
    "#,
        )
        .eval()
        .unwrap();
    assert!(!result.0.is_empty());
    assert!(!result.1.is_empty());
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

#[test]
fn test_has_prefix() {
    let engine = LuaEngine::new().unwrap();
    let result: bool = engine
        .lua
        .load(r#"return vmrHasPrefix("hello", "he")"#)
        .eval()
        .unwrap();
    assert!(result);
    let result: bool = engine
        .lua
        .load(r#"return vmrHasPrefix("hello", "xx")"#)
        .eval()
        .unwrap();
    assert!(!result);
}

#[test]
fn test_has_suffix() {
    let engine = LuaEngine::new().unwrap();
    let result: bool = engine
        .lua
        .load(r#"return vmrHasSuffix("hello", "lo")"#)
        .eval()
        .unwrap();
    assert!(result);
    let result: bool = engine
        .lua
        .load(r#"return vmrHasSuffix("hello", "xx")"#)
        .eval()
        .unwrap();
    assert!(!result);
}

#[test]
fn test_contains() {
    let engine = LuaEngine::new().unwrap();
    let result: bool = engine
        .lua
        .load(r#"return vmrContains("abc", "a")"#)
        .eval()
        .unwrap();
    assert!(result);
    let result: bool = engine
        .lua
        .load(r#"return vmrContains("abc", "x")"#)
        .eval()
        .unwrap();
    assert!(!result);
}

#[test]
fn test_trim_prefix() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrTrimPrefix("hello", "he")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "llo");
    // prefix not present → original string returned
    let result: String = engine
        .lua
        .load(r#"return vmrTrimPrefix("hello", "xx")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_trim_suffix() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrTrimSuffix("hello", "lo")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "hel");
}

#[test]
fn test_trim() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrTrim("ddabcdd", "d")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "abc");
}

#[test]
fn test_trim_space() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrTrimSpace("  hello  ")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_to_lower() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrToLower("HELLO")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_len_string() {
    let engine = LuaEngine::new().unwrap();
    let result: i64 = engine
        .lua
        .load(r#"return vmrLenString("hello")"#)
        .eval()
        .unwrap();
    assert_eq!(result, 5);
}

// ---------------------------------------------------------------------------
// Split / Sprintf
// ---------------------------------------------------------------------------

#[test]
fn test_split() {
    let engine = LuaEngine::new().unwrap();
    let (table, len): (mlua::Table, i64) = engine
        .lua
        .load(r#"return vmrSplit("a,b,c", ",")"#)
        .eval()
        .unwrap();
    assert_eq!(len, 3);
    let first: String = table.get(1).unwrap();
    assert_eq!(first, "a");
    let second: String = table.get(2).unwrap();
    assert_eq!(second, "b");
}

#[test]
fn test_sprintf() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrSprintf("hello %s", {"world"})"#)
        .eval()
        .unwrap();
    assert!(result.contains("world"));
}

#[test]
fn test_sprintf_positional() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrSprintf("hello %1 %2", {"foo", "bar"})"#)
        .eval()
        .unwrap();
    assert_eq!(result, "hello foo bar");
}

// ---------------------------------------------------------------------------
// URL / Path join
// ---------------------------------------------------------------------------

#[test]
fn test_url_join() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrUrlJoin("https://example.com/v1", "path")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "https://example.com/v1/path");
}

#[test]
fn test_path_join() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrPathJoin("/home", "test")"#)
        .eval()
        .unwrap();
    assert!(result.contains("home"));
    assert!(result.contains("test"));
}

// ---------------------------------------------------------------------------
// Environment variables
// ---------------------------------------------------------------------------

#[test]
fn test_get_set_os_env() {
    let engine = LuaEngine::new().unwrap();
    let _: bool = engine
        .lua
        .load(r#"return vmrSetOsEnv("VMR_TEST_KEY", "test_value")"#)
        .eval()
        .unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrGetOsEnv("VMR_TEST_KEY")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "test_value");
}

// ---------------------------------------------------------------------------
// File I/O
// ---------------------------------------------------------------------------

#[test]
fn test_write_read_file() {
    let tmp = std::env::temp_dir().join("vmr_test_file.txt");
    let tmp_str = tmp.to_str().unwrap();

    let engine = LuaEngine::new().unwrap();
    let write_ok: bool = engine
        .lua
        .load(&format!(r#"return vmrWriteFile("{}", "hello world")"#, tmp_str))
        .eval()
        .unwrap();
    assert!(write_ok);

    let result: String = engine
        .lua
        .load(&format!(r#"return vmrReadFile("{}")"#, tmp_str))
        .eval()
        .unwrap();
    assert_eq!(result, "hello world");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_create_remove_dir() {
    let tmp = std::env::temp_dir().join("vmr_test_dir");
    let tmp_str = tmp.to_str().unwrap();

    let engine = LuaEngine::new().unwrap();
    let ok: bool = engine
        .lua
        .load(&format!(r#"return vmrCreateDir("{}")"#, tmp_str))
        .eval()
        .unwrap();
    assert!(ok);
    assert!(tmp.exists());

    let ok: bool = engine
        .lua
        .load(&format!(r#"return vmrRemoveAll("{}")"#, tmp_str))
        .eval()
        .unwrap();
    assert!(ok);
    assert!(!tmp.exists());
}

// ---------------------------------------------------------------------------
// Regex
// ---------------------------------------------------------------------------

#[test]
fn test_regexp_find_string() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrRegexpFindString("r(.+)d", "hello regexp world")"#)
        .eval()
        .unwrap();
    assert!(result.contains("regexp"));
}

#[test]
fn test_regexp_no_match_returns_empty() {
    let engine = LuaEngine::new().unwrap();
    let result: String = engine
        .lua
        .load(r#"return vmrRegexpFindString("zzz", "hello world")"#)
        .eval()
        .unwrap();
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// Proxy
// ---------------------------------------------------------------------------

#[test]
fn test_get_proxy_no_config() {
    let engine = LuaEngine::new().unwrap();
    let result: (String, String, String) = engine
        .lua
        .load(r#"return vmrGetProxy()"#)
        .eval()
        .unwrap();
    // Proxy may or may not be configured — just verify types are valid
    assert!(result.2.parse::<u16>().is_ok() || result.2 == "0"); // port should be numeric or "0"
}

// ---------------------------------------------------------------------------
// Version List
// ---------------------------------------------------------------------------

#[test]
fn test_new_version_list() {
    let engine = LuaEngine::new().unwrap();
    let vl: mlua::Table = engine
        .lua
        .load(r#"return vmrNewVersionList()"#)
        .eval()
        .unwrap();
    // Should be an empty table
    assert_eq!(vl.raw_len(), 0);
}

#[test]
fn test_add_item_to_version_list() {
    let engine = LuaEngine::new().unwrap();
    engine
        .lua
        .load(
            r#"
        vl = vmrNewVersionList()
        item = {
            ["url"] = "https://example.com/sdk.zip",
            ["arch"] = "amd64",
            ["os"] = "linux",
            ["sum"] = "abc123",
            ["sum_type"] = "sha256",
            ["size"] = 1024,
            ["installer"] = "unarchiver",
            ["lts"] = "",
            ["extra"] = ""
        }
        vl = vmrAddItem(vl, "v1.0.0", item)
    "#,
        )
        .exec()
        .unwrap();
}

#[test]
fn test_merge_version_lists() {
    let engine = LuaEngine::new().unwrap();
    let merged: mlua::Table = engine
        .lua
        .load(
            r#"
        vl1 = vmrNewVersionList()
        item1 = {["url"]="https://a.com", ["arch"]="amd64", ["os"]="linux", ["installer"]="unarchiver"}
        vmrAddItem(vl1, "v1.0.0", item1)
        
        vl2 = vmrNewVersionList()
        item2 = {["url"]="https://b.com", ["arch"]="amd64", ["os"]="linux", ["installer"]="unarchiver"}
        vmrAddItem(vl2, "v2.0.0", item2)
        
        return vmrMergeVersionList(vl1, vl2)
    "#,
        )
        .eval()
        .unwrap();

    // Both versions should be present
    let v1: mlua::Table = merged.get("v1.0.0").unwrap();
    assert!(v1.raw_len() >= 1);
    let v2: mlua::Table = merged.get("v2.0.0").unwrap();
    assert!(v2.raw_len() >= 1);
}

// ---------------------------------------------------------------------------
// Installer Config
// ---------------------------------------------------------------------------

#[test]
fn test_new_installer_config() {
    let engine = LuaEngine::new().unwrap();
    let ic: mlua::Table = engine
        .lua
        .load(r#"return vmrNewInstallerConfig()"#)
        .eval()
        .unwrap();

    // The table should have the expected sub-tables and fields
    let flag_files: mlua::Table = ic.get("flag_files").unwrap();
    assert_eq!(flag_files.raw_len(), 0);

    let flag_dir_excepted: bool = ic.get("flag_dir_excepted").unwrap();
    assert!(!flag_dir_excepted);

    let binary_dirs: mlua::Table = ic.get("binary_dirs").unwrap();
    assert_eq!(binary_dirs.raw_len(), 0);
}

// ---------------------------------------------------------------------------
// Exec system command
// ---------------------------------------------------------------------------

#[test]
fn test_exec_system_cmd_echo() {
    let engine = LuaEngine::new().unwrap();
    // vmrExecSystemCmd(collect, workdir, cmd, arg1, arg2, ...) -> (output, success)
    let (output, success): (String, bool) = engine
        .lua
        .load(r#"return vmrExecSystemCmd(true, "", "echo", "hello")"#)
        .eval()
        .unwrap();
    assert!(success);
    assert_eq!(output, "hello");
}

// ---------------------------------------------------------------------------
// Copy file
// ---------------------------------------------------------------------------

#[test]
fn test_copy_file() {
    let tmp_dir = std::env::temp_dir().join("vmr_test_copy");
    let src_path = tmp_dir.join("src.txt");
    let dst_path = tmp_dir.join("dst.txt");
    let src_str = src_path.to_str().unwrap();
    let dst_str = dst_path.to_str().unwrap();

    // Ensure temp dir exists
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let _ = std::fs::remove_file(&dst_path);

    let engine = LuaEngine::new().unwrap();

    // Write source file
    let _: bool = engine
        .lua
        .load(&format!(r#"return vmrWriteFile("{}", "copy-test")"#, src_str))
        .eval()
        .unwrap();

    // Copy
    let ok: bool = engine
        .lua
        .load(&format!(r#"return vmrCopyFile("{}", "{}")"#, src_str, dst_str))
        .eval()
        .unwrap();
    assert!(ok);

    let content = std::fs::read_to_string(&dst_path).unwrap();
    assert_eq!(content, "copy-test");

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);
}
