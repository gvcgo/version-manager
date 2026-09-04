//! gjson 绑定（vmrInitGJson/GetString/GetInt/GetByKey/MapEach/GetByIndex/SliceEach）。
//!
//! Go 用 gjson 的 `Json.Get(path)`；Rust 侧自实现**点路径子集**：
//! 段为对象键或数组下标（数字段），支持 `a.b.0.c`（对齐 gjson 的 `.` 数组下标）。
//! userdata 内为 serde_json::Value；MapEach/SliceEach 回调值同为 JsonUD（Value），
//! 供继续按路径取。

use mlua::{Function, Lua, UserData, Value};

use crate::bindings::register_fn;
use crate::req::StringUD;

#[derive(Clone)]
pub struct JsonUD(pub serde_json::Value);
impl UserData for JsonUD {}

fn init_json(ud: &Value) -> serde_json::Value {
    let s: String = match ud {
        Value::UserData(u) => u
            .borrow::<StringUD>()
            .map(|x| x.0.clone())
            .unwrap_or_default(),
        Value::String(s) => crate::req::lua_string_to_owned(s),
        _ => String::new(),
    };
    serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
}

fn get_json(ud: &Value) -> Option<serde_json::Value> {
    match ud {
        Value::UserData(u) => u.borrow::<JsonUD>().ok().map(|j| j.0.clone()),
        _ => None,
    }
}

/// 点路径解析：对象键 / 数组下标数字段。
pub fn get_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(root);
    }
    let mut cur = root;
    for seg in path.split('.') {
        match cur {
            serde_json::Value::Object(map) => {
                cur = map.get(seg)?;
            }
            serde_json::Value::Array(arr) => {
                let i: usize = seg.parse().ok()?;
                cur = arr.get(i)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

/// 值字符串化（对齐 gconv.String 的常用情形）。
pub fn json_to_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                // Go fmt %v 打印浮点：整值不带小数点。
                if f == f.trunc() && f.is_finite() && f.abs() < 1e15 {
                    (f as i64).to_string()
                } else {
                    f.to_string()
                }
            } else {
                String::new()
            }
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn as_i64(v: &serde_json::Value) -> i64 {
    match v {
        serde_json::Value::Number(n) => n
            .as_i64()
            .unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        serde_json::Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(lua, "vmrInitGJson", |lua, (ud,): (Value,)| {
        let v = init_json(&ud);
        lua.create_userdata(JsonUD(v)).map(Value::UserData)
    })?;

    register_fn(lua, "vmrGetString", |_, (ud, path): (Value, String)| {
        let v = get_json(&ud);
        let out = v
            .as_ref()
            .and_then(|root| get_path(root, &path))
            .map(json_to_str)
            .unwrap_or_default();
        Ok(out)
    })?;

    register_fn(lua, "vmrGetInt", |_, (ud, path): (Value, String)| {
        let v = get_json(&ud);
        let out = v
            .as_ref()
            .and_then(|r| get_path(r, &path))
            .map(as_i64)
            .unwrap_or(0);
        Ok(out)
    })?;

    register_fn(
        lua,
        "vmrGetByKey",
        |_, (ud, path, key): (Value, String, String)| {
            let v = get_json(&ud);
            let out = v
                .as_ref()
                .and_then(|r| get_path(r, &path))
                .and_then(|obj| obj.get(&key))
                .map(json_to_str)
                .unwrap_or_default();
            Ok(out)
        },
    )?;

    register_fn(
        lua,
        "vmrMapEach",
        |lua, (ud, path, cb): (Value, String, Function)| {
            let v = get_json(&ud);
            let Some(target) = v.as_ref().and_then(|r| get_path(r, &path)) else {
                return Ok(());
            };
            let serde_json::Value::Object(map) = target else {
                return Ok(());
            };
            for (k, val) in map {
                let item = lua.create_userdata(JsonUD(val.clone()))?;
                cb.call::<()>((k.as_str(), item))?;
            }
            Ok(())
        },
    )?;

    register_fn(
        lua,
        "vmrSliceEach",
        |lua, (ud, path, cb): (Value, String, Function)| {
            let v = get_json(&ud);
            let Some(target) = v.as_ref().and_then(|r| get_path(r, &path)) else {
                return Ok(());
            };
            let serde_json::Value::Array(arr) = target else {
                return Ok(());
            };
            for (idx, val) in arr.iter().enumerate() {
                if val.is_null() {
                    continue;
                }
                let item = lua.create_userdata(JsonUD(val.clone()))?;
                cb.call::<()>((idx as i64 + 1, item))?;
            }
            Ok(())
        },
    )?;

    register_fn(
        lua,
        "vmrGetByIndex",
        |_, (ud, path, index): (Value, String, i64)| {
            let v = get_json(&ud);
            let out = v
                .as_ref()
                .and_then(|r| get_path(r, &path))
                .and_then(|obj| {
                    let arr = obj.as_array()?;
                    if index < 1 {
                        return None;
                    }
                    arr.get((index - 1) as usize)
                })
                .map(json_to_str)
                .unwrap_or_default();
            Ok(out)
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn path_resolution() {
        let v = json!({"metadata": {"versioning": {"versions": {"version": ["1.0", "2.0"]}}}});
        assert!(get_path(&v, "metadata.versioning.versions.version").is_some());
        assert_eq!(
            json_to_str(get_path(&v, "metadata.versioning.versions.version.1").unwrap()),
            "2.0"
        );
        assert_eq!(
            json_to_str(get_path(&v, "nope").unwrap_or(&serde_json::Value::Null)),
            ""
        );
    }

    #[test]
    fn number_to_str() {
        assert_eq!(json_to_str(&json!(1.0)), "1");
        assert_eq!(json_to_str(&json!(1.5)), "1.5");
        assert_eq!(json_to_str(&json!("x")), "x");
    }
}
