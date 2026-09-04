//! Bindings: `vmrGetResponse` / `vmrGetProxy` (mirroring Go `lua_global/req.go`, `proxy.go`).
//!
//! Key semantics (Go quirk): `vmrGetResponse` **returns a userdata-wrapped string**
//! for `vmrInitSelection` / `vmrInitGJson` to unwrap; on failure it returns an empty-string userdata.
//! The proxy only reads the `VCOLLECTOR_PROXY` env (truthy boolean parsing); it does not go through the mirror/reverse-proxy chain.

use std::time::Duration;

use mlua::{Lua, Table, UserData, Value};

use crate::bindings::register_fn;

pub const PROXY_ENV_NAME: &str = "VCOLLECTOR_PROXY";
const DEFAULT_TIMEOUT: u64 = 180;

/// Return wrapper of vmrGetResponse (the Go userdata holds a string).
#[derive(Clone)]
pub struct StringUD(pub String);
impl UserData for StringUD {}

fn env_bool(key: &str) -> bool {
    std::env::var(key)
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "on"))
        .unwrap_or(false)
}

fn proxy_from_env() -> Option<String> {
    if env_bool(PROXY_ENV_NAME) {
        std::env::var(PROXY_ENV_NAME).ok()
    } else {
        None
    }
}

/// GET request body; on failure → Err (the caller treats it as an empty string, matching the Go resp=="" branch).
fn http_get(url: &str, timeout_secs: u64, headers: &[(String, String)]) -> Result<String, String> {
    let mut builder = reqwest::blocking::Client::builder().user_agent("");
    if let Some(p) = proxy_from_env() {
        if let Ok(proxy) = reqwest::Proxy::all(&p) {
            builder = builder.proxy(proxy);
        }
    }
    let client = builder.build().map_err(|e| e.to_string())?;
    let mut req = client
        .get(url)
        .timeout(Duration::from_secs(timeout_secs.max(1)));
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("http {}", resp.status()));
    }
    let body = resp.text().map_err(|e| e.to_string())?;
    if body.is_empty() {
        return Err("empty body".to_string());
    }
    Ok(body)
}

/// Parses a proxy URI → (scheme, host, port); empty/invalid returns Go semantics ("","","0").
fn split_proxy_uri(s: &str) -> (String, String, String) {
    let Some((scheme, rest)) = s.split_once("://") else {
        return (String::new(), String::new(), "0".to_string());
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    match authority.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (scheme.to_string(), h.to_string(), p.to_string())
        }
        _ => (scheme.to_string(), authority.to_string(), String::new()),
    }
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(
        lua,
        "vmrGetResponse",
        |_, (url, timeout, headers_tbl): (String, i64, Option<Table>)| {
            let timeout = if timeout > 0 {
                timeout as u64
            } else {
                DEFAULT_TIMEOUT
            };
            let mut headers: Vec<(String, String)> = Vec::new();
            if let Some(t) = headers_tbl {
                for (k, v) in t.pairs::<Value, Value>().flatten() {
                    headers.push((str_of(&k), str_of(&v)));
                }
            }
            let body = http_get(&url, timeout, &headers).unwrap_or_default();
            Ok(StringUD(body))
        },
    )?;

    register_fn(lua, "vmrGetProxy", |_, ()| {
        let cfg = vmr_core::conf::VMRConf::new().proxy_uri;
        let (scheme, host, port) = if cfg.is_empty() {
            (String::new(), String::new(), "0".to_string())
        } else {
            split_proxy_uri(&cfg)
        };
        Ok((scheme, host, port))
    })?;
    Ok(())
}

/// Lua value → string (matching gopher-lua `LValue.String()`).
pub fn lua_string_to_owned(s: &mlua::String) -> String {
    s.to_string_lossy()
}

pub fn str_of(v: &Value) -> String {
    match v {
        Value::String(s) => lua_string_to_owned(s),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Nil => String::new(),
        Value::Table(_) => "<table>".to_string(),
        Value::UserData(_) => "<userdata>".to_string(),
        Value::Function(_) => "<function>".to_string(),
        other => format!("{other:?}"),
    }
}

/// Reads a table field (missing → "", mirroring Go GetStringFromLTable).
pub fn table_str(t: &Table, key: &str) -> mlua::Result<String> {
    let v: Value = t.get(key)?;
    Ok(str_of(&v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_of_variants() {
        assert_eq!(str_of(&Value::Integer(12)), "12");
        assert_eq!(str_of(&Value::Boolean(true)), "true");
        assert_eq!(str_of(&Value::Nil), "");
    }

    #[test]
    fn proxy_split() {
        assert_eq!(
            split_proxy_uri("http://127.0.0.1:7890"),
            (
                "http".to_string(),
                "127.0.0.1".to_string(),
                "7890".to_string()
            )
        );
        assert_eq!(
            split_proxy_uri(""),
            (String::new(), String::new(), "0".to_string())
        );
        assert_eq!(split_proxy_uri("socks5://h:1080").0, "socks5");
    }
}
