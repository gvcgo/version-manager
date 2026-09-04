//! 版本列表绑定（vmrNewVersionList/AddItem/MergeVersionList），对齐 Go
//! `lua_global/version.go`：crawl 内以 userdata 持有 VersionList，
//! AddItem 返回同一列表（Lua 链式赋值），merge 原地并入后者。

use mlua::{Lua, Table, UserData, Value};

use crate::bindings::register_fn;
use crate::req::{str_of, table_str};
use crate::types::{Item, VersionList};

#[derive(Clone)]
pub struct VersionListUD(pub VersionList);
impl UserData for VersionListUD {}

fn read_item(t: &Table) -> mlua::Result<Item> {
    Ok(Item {
        url: table_str(t, "url")?,
        arch: table_str(t, "arch")?,
        os: table_str(t, "os")?,
        sum: table_str(t, "sum")?,
        sum_type: table_str(t, "sum_type")?,
        size: parse_size(&t.get::<Value>("size")?),
        installer: table_str(t, "installer")?,
        lts: table_str(t, "lts")?,
        extra: table_str(t, "extra")?,
    })
}

fn parse_size(v: &Value) -> i64 {
    match v {
        Value::Nil => 0,
        Value::Integer(i) => *i,
        Value::Number(n) => *n as i64,
        Value::String(s) => s.to_string_lossy().parse().unwrap_or(0),
        _ => 0,
    }
}

fn push_result(lua: &Lua, vl: VersionListUD) -> mlua::Result<Value> {
    lua.create_userdata(vl).map(Value::UserData)
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(lua, "vmrNewVersionList", |lua, ()| {
        push_result(lua, VersionListUD(VersionList::new()))
    })?;

    register_fn(
        lua,
        "vmrAddItem",
        |_lua, (ud, ver, item_tbl): (AnyUserData, String, Option<Table>)| {
            let mut guard = ud.borrow_mut::<VersionListUD>()?;
            let vl = &mut guard.0;
            if !ver.is_empty() {
                let items = vl.entry(ver).or_default();
                if let Some(t) = item_tbl {
                    let item = read_item(&t)?;
                    items.push(item);
                }
            }
            drop(guard);
            Ok(Value::UserData(ud))
        },
    )?;

    register_fn(
        lua,
        "vmrMergeVersionList",
        |_lua, (ud1, ud2): (AnyUserData, AnyUserData)| {
            let mut guard = ud1.borrow_mut::<VersionListUD>()?;
            let second = ud2.borrow::<VersionListUD>()?;
            for (k, v) in &second.0 {
                let items = guard.0.entry(k.clone()).or_default();
                items.extend(v.clone());
            }
            drop(guard);
            Ok(Value::UserData(ud1))
        },
    )?;

    // 兼容别名：从 Value 形式收尾（确保 Lua nil 参数不 panic —— Go 会返回 nil ud）。
    register_fn(lua, "_vmrAddItemSafe", |_, ()| Ok(()))?;
    Ok(())
}

/// 供 plugin 读取 crawl 返回值（userdata → VersionList）。
pub fn vl_from_value(v: &Value) -> Option<VersionList> {
    match v {
        Value::UserData(u) => u.borrow::<VersionListUD>().ok().map(|g| g.0.clone()),
        _ => None,
    }
}

pub fn vl_to_value(lua: &Lua, vl: VersionList) -> mlua::Result<Value> {
    push_result(lua, VersionListUD(vl))
}

type AnyUserData = mlua::AnyUserData;

/// 过滤出当前平台条目 → map[版本]最后匹配 Item（对齐 Go GetSDKVersions）。
pub fn filter_current_platform(
    vl: &VersionList,
    os: &str,
    arch: &str,
) -> std::collections::HashMap<String, Item> {
    let mut out = std::collections::HashMap::new();
    for (ver, items) in vl {
        for item in items {
            if item.os == os && item.arch == arch {
                out.insert(ver.clone(), item.clone());
            }
        }
    }
    out
}

// 供测试与提示使用（避免未使用告警）。
#[allow(dead_code)]
fn _unused(_: &str) {}

/// 辅助：把 Lua 值里的字符串转成 Item 列表外的散件时用到（见 plugin）。
#[allow(dead_code)]
pub fn str_of_val(v: &Value) -> String {
    str_of(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_keeps_last_match() {
        let mut vl = VersionList::new();
        vl.insert(
            "1.0".into(),
            vec![
                Item {
                    os: "linux".into(),
                    arch: "amd64".into(),
                    installer: "unarchiver".into(),
                    ..Default::default()
                },
                Item {
                    os: "darwin".into(),
                    arch: "amd64".into(),
                    installer: "unarchiver".into(),
                    ..Default::default()
                },
            ],
        );
        let out = filter_current_platform(&vl, "linux", "amd64");
        assert_eq!(out.len(), 1);
        assert_eq!(out["1.0"].os, "linux");
    }
}
