//! Central hub for registering the vmr* globals (mirrors Go `lua_global/lua.go` init plus constant injection).

use mlua::{FromLuaMulti, IntoLuaMulti, Lua};

/// Convenience registration: `globals().set(name, lua.create_function(f)?)`.
pub(crate) fn register_fn<A, R, F>(lua: &Lua, name: &str, f: F) -> mlua::Result<()>
where
    A: FromLuaMulti + 'static,
    R: IntoLuaMulti + 'static,
    F: Fn(&Lua, A) -> mlua::Result<R> + 'static,
{
    lua.globals().set(name, lua.create_function(f)?)
}

/// Registers the installer constants missing on the Go side that plugins reference (plan.md §1).
fn register_installer_consts(lua: &Lua) -> mlua::Result<()> {
    use crate::types::installer_kind::*;
    for (name, val) in [
        ("vmrInstallerUnarchiver", UNARCHIVER),
        ("vmrInstallerExecutable", EXECUTABLE),
        ("vmrInstallerConda", CONDA),
        ("vmrInstallerCondaForge", CONDA_FORGE),
        ("vmrInstallerCoursier", COURSIER),
        ("vmrInstallerDpkg", DPKG),
        ("vmrInstallerRpm", RPM),
    ] {
        lua.globals().set(name, val)?;
    }
    Ok(())
}

/// Builds a runtime and registers every vmr* function and constant (mirrors Go `Lua.NewLua().init()`).
pub fn new_runtime() -> mlua::Result<Lua> {
    let lua = Lua::new();
    // Guard: script errors surface as Error (do not panic the process).
    let _ = lua.set_memory_limit(512 * 1024 * 1024);
    crate::req::register(&lua)?;
    crate::html::register(&lua)?;
    crate::json::register(&lua)?;
    crate::utils_bind::register(&lua)?;
    crate::version::register(&lua)?;
    crate::github_bind::register(&lua)?;
    crate::installer_conf::register(&lua)?;
    crate::conda_bridge::register(&lua)?;
    register_installer_consts(&lua)?;
    Ok(lua)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_boot_and_simple_script() {
        let lua = new_runtime().unwrap();
        // After registration, 50+ globals exist; a few key ones must be present.
        for name in [
            "vmrGetOsArch",
            "vmrGetResponse",
            "vmrNewVersionList",
            "vmrInstallerUnarchiver",
        ] {
            let v: mlua::Value = lua.globals().get(name).unwrap();
            assert!(!matches!(v, mlua::Value::Nil), "{name} missing");
        }
        // Constant value is correct.
        let c: String = lua.globals().get("vmrInstallerUnarchiver").unwrap();
        assert_eq!(c, "unarchiver");
        // A simple script calling the version function chain.
        lua.load(
            r#"
            vl = vmrNewVersionList()
            item = { url = "x", os = "linux", arch = "amd64" }
            vl = vmrAddItem(vl, "1.2.3", item)
            "#,
        )
        .exec()
        .unwrap();
        let t: mlua::Table = lua.globals().get("item").unwrap();
        let os: String = t.get("os").unwrap();
        assert_eq!(os, "linux");
        let vlv: mlua::Value = lua.globals().get("vl").unwrap();
        assert!(matches!(vlv, mlua::Value::UserData(_)), "vl 应为 userdata");
    }
}
