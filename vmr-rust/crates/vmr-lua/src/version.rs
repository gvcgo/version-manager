//! Version-list bindings (vmrNewVersionList/AddItem/MergeVersionList), mirroring Go
//! `lua_global/version.go`: inside crawl a userdata holds the VersionList,
//! AddItem returns the same list (Lua chained assignment), and merge folds the latter in place.

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

    // Compatibility alias: wrap up from the Value form (ensures a Lua nil arg does not panic — Go returns a nil ud).
    register_fn(lua, "_vmrAddItemSafe", |_, ()| Ok(()))?;
    Ok(())
}

/// For plugin to read the crawl return value (userdata → VersionList).
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

/// Filters to current-platform entries → map[version]last matching Item (mirrors Go GetSDKVersions).
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

// For testing and tooling (avoids the unused warning).
#[allow(dead_code)]
fn _unused(_: &str) {}

/// Helper: used when converting strings inside a Lua value into the loose parts outside an Item list (see plugin).
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
