//! vmr* 全局注册中枢（对齐 Go `lua_global/lua.go` 的 init + 常量注入）。

use mlua::{FromLuaMulti, IntoLuaMulti, Lua};

/// 便捷注册：`globals().set(name, lua.create_function(f)?)`。
pub(crate) fn register_fn<A, R, F>(lua: &Lua, name: &str, f: F) -> mlua::Result<()>
where
    A: FromLuaMulti + 'static,
    R: IntoLuaMulti + 'static,
    F: Fn(&Lua, A) -> mlua::Result<R> + 'static,
{
    lua.globals().set(name, lua.create_function(f)?)
}

/// 注册 Go 侧缺失、插件引用到的 installer 常量（plan.md §1）。
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

/// 建运行时并注册全部 vmr* 函数与常量（对齐 Go `Lua.NewLua().init()`）。
pub fn new_runtime() -> mlua::Result<Lua> {
    let lua = Lua::new();
    // 防御：脚本错误暴露为 Error（不 panic 进程）。
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
        // 注册后 50+ 全局应存在关键几个。
        for name in [
            "vmrGetOsArch",
            "vmrGetResponse",
            "vmrNewVersionList",
            "vmrInstallerUnarchiver",
        ] {
            let v: mlua::Value = lua.globals().get(name).unwrap();
            assert!(!matches!(v, mlua::Value::Nil), "{name} missing");
        }
        // 常量值正确。
        let c: String = lua.globals().get("vmrInstallerUnarchiver").unwrap();
        assert_eq!(c, "unarchiver");
        // 简单脚本调用版本函数链。
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
