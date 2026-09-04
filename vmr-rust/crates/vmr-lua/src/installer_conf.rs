//! 安装配置绑定（vmrNewInstallerConfig/AddFlagFiles/EnableFlagDirExcepted/
//! AddBinaryDirs/AddAdditionalEnvs）与全局 `ic` 读取。
//!
//! 对齐 Go `lua_global/installer.go`：链式调用返回同一 userdata；os 参数
//! 空串时同时追加三平台；不认识的 os 原样返回（不改动）。

use mlua::{Lua, Table, UserData, Value};

use crate::bindings::register_fn;
use crate::types::{AdditionalEnv, DirPath, InstallerConfig};

#[derive(Clone)]
pub struct ICUD(pub InstallerConfig);
impl UserData for ICUD {}

fn push_ic(lua: &Lua, ic: ICUD) -> mlua::Result<Value> {
    lua.create_userdata(ic).map(Value::UserData)
}

fn path_list(t: Option<Table>) -> mlua::Result<DirPath> {
    let mut out = Vec::new();
    if let Some(t) = t {
        for v in t.sequence_values::<Value>().flatten() {
            out.push(crate::req::str_of(&v));
        }
    }
    Ok(out)
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(lua, "vmrNewInstallerConfig", |lua, ()| {
        push_ic(lua, ICUD(InstallerConfig::new()))
    })?;

    register_fn(
        lua,
        "vmrAddFlagFiles",
        |_lua, (ud, os, list): (AnyUserData, String, Option<Table>)| {
            let mut g = ud.borrow_mut::<ICUD>()?;
            let ic = &mut g.0;
            let files = ic.flag_files.get_or_insert_with(Default::default);
            let vals = path_list(list)?;
            match os.as_str() {
                "" => {
                    files.windows.extend(vals.iter().cloned());
                    files.linux.extend(vals.iter().cloned());
                    files.darwin.extend(vals.iter().cloned());
                }
                "windows" => files.windows.extend(vals),
                "linux" => files.linux.extend(vals),
                "darwin" => files.darwin.extend(vals),
                _ => return Ok(Value::UserData(ud)),
            }
            drop(g);
            Ok(Value::UserData(ud))
        },
    )?;

    register_fn(
        lua,
        "vmrEnableFlagDirExcepted",
        |_lua, (ud,): (AnyUserData,)| {
            let mut g = ud.borrow_mut::<ICUD>()?;
            g.0.flag_dir_excepted = true;
            drop(g);
            Ok(Value::UserData(ud))
        },
    )?;

    register_fn(
        lua,
        "vmrAddBinaryDirs",
        |_lua, (ud, os, list): (AnyUserData, String, Option<Table>)| {
            let mut g = ud.borrow_mut::<ICUD>()?;
            let ic = &mut g.0;
            let dirs = ic.binary_dirs.get_or_insert_with(Default::default);
            let one = path_list(list)?;
            if one.is_empty() {
                return Ok(Value::UserData(ud));
            }
            match os.as_str() {
                "" => {
                    dirs.windows.push(one.clone());
                    dirs.linux.push(one.clone());
                    dirs.darwin.push(one);
                }
                "windows" => dirs.windows.push(one),
                "linux" => dirs.linux.push(one),
                "darwin" => dirs.darwin.push(one),
                _ => return Ok(Value::UserData(ud)),
            }
            drop(g);
            Ok(Value::UserData(ud))
        },
    )?;

    register_fn(
        lua,
        "vmrAddAdditionalEnvs",
        |_lua, (ud, name, list, ver): (AnyUserData, String, Option<Table>, String)| {
            if name.is_empty() {
                return Ok(Value::UserData(ud));
            }
            let mut g = ud.borrow_mut::<ICUD>()?;
            let p = path_list(list)?;
            g.0.additional_envs.push(AdditionalEnv {
                name,
                value: vec![p],
                version: ver,
            });
            drop(g);
            Ok(Value::UserData(ud))
        },
    )?;
    Ok(())
}

/// 读全局 `ic`（脚本执行后调用），对齐 Go `GetInstallerConfig`。
pub fn ic_from_global(lua: &Lua) -> Option<InstallerConfig> {
    let v: Value = lua.globals().get("ic").ok()?;
    match v {
        Value::UserData(u) => u.borrow::<ICUD>().ok().map(|g| g.0.clone()),
        _ => None,
    }
}

pub fn ic_to_value(lua: &Lua, ic: InstallerConfig) -> mlua::Result<Value> {
    push_ic(lua, ICUD(ic))
}

type AnyUserData = mlua::AnyUserData;
