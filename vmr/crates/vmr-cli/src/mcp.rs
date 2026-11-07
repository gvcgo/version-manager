//! MCP (Model Context Protocol) server over stdio.
//!
//! Implements a dual-era subset of the protocol: the stateless 2026-07-28 revision
//! (`server/discover`, per-request `_meta`, `resultType`) plus the legacy
//! `initialize` handshake used by 2025-06-18 and earlier clients. Only stdio is
//! supported; every message is one JSON object per line.
//!
//! Stdout carries JSON-RPC exclusively. Tool implementations are the same data
//! functions the CLI uses and never print; child processes spawned by the installer
//! and Lua plugins are guarded in `vmr-installer` / `vmr-utils` via `VMR_MCP_SERVER`.

use std::io::{BufRead, Write};

use serde_json::{Map, Value, json};

use crate::{InstallSummary, PluginRow, SdkVersions, VersionRow};

const MODERN_VERSION: &str = "2026-07-28";
const LEGACY_VERSIONS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];
const SERVER_NAME: &str = "vmr";
const SERVER_INSTRUCTIONS: &str = "vmr manages SDK versions on this machine. Read-only tools \
discover plugins, available versions, installed versions and the environment of the current \
version. vmr_install and vmr_uninstall change global state: installs write to ~/.vmr and update \
the shell environment for new shells, but cannot change the environment of the MCP client \
itself. Use vmr_env to obtain PATH/env entries when running SDK binaries from the client.";

pub(crate) fn serve() -> Result<(), String> {
    // Guard child processes: `vmr-installer` / `vmr-utils` must not let them write to the
    // JSON-RPC stdout or consume the request stream on stdin.
    unsafe { std::env::set_var("VMR_MCP_SERVER", "1") };

    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("read stdin failed: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let message: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                write_message(
                    &mut out,
                    &rpc_error(
                        Value::Null,
                        -32700,
                        "Parse error",
                        Some(json!({"detail": e.to_string()})),
                    ),
                );
                continue;
            }
        };
        let reply = match message {
            Value::Array(items) => {
                let replies: Vec<Value> = items.iter().filter_map(handle_message).collect();
                if replies.is_empty() {
                    None
                } else {
                    Some(Value::Array(replies))
                }
            }
            msg => handle_message(&msg),
        };
        if let Some(reply) = reply {
            write_message(&mut out, &reply);
        }
    }
    Ok(())
}

fn write_message(out: &mut impl Write, message: &Value) {
    if serde_json::to_writer(&mut *out, message).is_ok() {
        let _ = out.write_all(b"\n");
        let _ = out.flush();
    }
}

struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>, data: Option<Value>) -> Self {
        RpcError {
            code,
            message: message.into(),
            data,
        }
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        RpcError::new(-32602, message, None)
    }
}

fn rpc_error(id: Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut error = json!({"code": code, "message": message});
    if let Some(data) = data {
        error["data"] = data;
    }
    json!({"jsonrpc": "2.0", "id": id, "error": error})
}

#[derive(Clone, Copy)]
struct Era {
    modern: bool,
}

fn handle_message(message: &Value) -> Option<Value> {
    let Some(object) = message.as_object() else {
        return Some(rpc_error(Value::Null, -32600, "Invalid Request", None));
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(rpc_error(Value::Null, -32600, "Invalid Request", None));
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        // A response to a server-initiated request; vmr never sends requests, so ignore it.
        if object.contains_key("result") || object.contains_key("error") {
            return None;
        }
        return Some(rpc_error(Value::Null, -32600, "Invalid Request", None));
    };
    // Notifications (initialized, cancelled, progress, ...) get no response.
    let id = object.get("id")?;
    if id.is_null() || !(id.is_string() || id.is_number()) {
        return Some(rpc_error(
            Value::Null,
            -32600,
            "Invalid Request",
            Some(json!({"detail": "id must be a string or number"})),
        ));
    }
    let id = id.clone();
    let params = object.get("params").cloned().unwrap_or(Value::Null);

    let era = match era_for(&params) {
        Ok(era) => era,
        Err(error) => return Some(rpc_error(id, error.code, &error.message, error.data)),
    };

    match dispatch(method, &params) {
        Ok(result) => {
            let result = finalize(era, method, result);
            Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
        }
        Err(error) => Some(rpc_error(id, error.code, &error.message, error.data)),
    }
}

/// Determines the protocol era from the per-request `_meta` and validates the modern
/// required fields. Requests without modern metadata are treated as legacy.
fn era_for(params: &Value) -> Result<Era, RpcError> {
    let Some(meta) = params.get("_meta").and_then(Value::as_object) else {
        return Ok(Era { modern: false });
    };
    let Some(version) = meta.get("io.modelcontextprotocol/protocolVersion") else {
        return Ok(Era { modern: false });
    };
    let version = version
        .as_str()
        .ok_or_else(|| RpcError::invalid_params("`_meta` protocolVersion must be a string"))?;
    match meta
        .get("io.modelcontextprotocol/clientCapabilities")
        .and_then(Value::as_object)
    {
        Some(_) => {}
        None => {
            return Err(RpcError::invalid_params(
                "`_meta` clientCapabilities object is required",
            ));
        }
    }
    if version != MODERN_VERSION {
        return Err(RpcError::new(
            -32022,
            "Unsupported protocol version",
            Some(json!({"supported": [MODERN_VERSION], "requested": version})),
        ));
    }
    Ok(Era { modern: true })
}

fn finalize(era: Era, method: &str, mut result: Value) -> Value {
    if !era.modern {
        return result;
    }
    result["resultType"] = json!("complete");
    result["_meta"] = json!({"io.modelcontextprotocol/serverInfo": server_info()});
    match method {
        "tools/list" => {
            result["ttlMs"] = json!(60_000);
            result["cacheScope"] = json!("public");
        }
        "server/discover" => {
            result["ttlMs"] = json!(3_600_000);
            result["cacheScope"] = json!("public");
        }
        _ => {}
    }
    result
}

fn server_info() -> Value {
    json!({"name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION")})
}

fn dispatch(method: &str, params: &Value) -> Result<Value, RpcError> {
    match method {
        "initialize" => Ok(initialize(params)),
        "server/discover" => Ok(json!({
            "resultType": "complete",
            "supportedVersions": [MODERN_VERSION],
            "capabilities": {"tools": {}},
            "instructions": SERVER_INSTRUCTIONS,
            "_meta": {"io.modelcontextprotocol/serverInfo": server_info()},
            "ttlMs": 3_600_000,
            "cacheScope": "public"
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => call_tool(params),
        other => Err(RpcError::new(
            -32601,
            format!("Method not found: {other}"),
            None,
        )),
    }
}

fn initialize(params: &Value) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("");
    let version = if LEGACY_VERSIONS.contains(&requested) {
        requested
    } else {
        // Latest legacy revision this server implements.
        "2025-06-18"
    };
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {}},
        "serverInfo": server_info()
    })
}

fn tools() -> Value {
    json!([
        {
            "name": "vmr_list_plugins",
            "title": "List vmr plugins",
            "description": "List the vmr Lua plugins installed on this machine and the SDKs they support.",
            "inputSchema": {"type": "object", "additionalProperties": false},
            "annotations": {"readOnlyHint": true}
        },
        {
            "name": "vmr_search_versions",
            "title": "Search SDK versions",
            "description": "List versions available for an SDK, either from its Lua plugin (filters by the current platform) or from the pixi/conda-forge channel.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sdk": {"type": "string", "description": "SDK name, e.g. go, node, python"},
                    "backend": {
                        "type": "string",
                        "enum": ["plugin", "pixi"],
                        "default": "plugin",
                        "description": "Version source: Lua plugin (default) or pixi/conda-forge"
                    },
                    "no_cache": {"type": "boolean", "default": false, "description": "Bypass cached version tables"}
                },
                "required": ["sdk"],
                "additionalProperties": false
            },
            "annotations": {"readOnlyHint": true, "openWorldHint": true}
        },
        {
            "name": "vmr_list_installed",
            "title": "List installed SDK versions",
            "description": "List installed SDK versions and the currently active version. Omit `sdk` to list every installed SDK.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sdk": {"type": "string", "description": "SDK name; omit for all installed SDKs"}
                },
                "additionalProperties": false
            },
            "annotations": {"readOnlyHint": true}
        },
        {
            "name": "vmr_install",
            "title": "Install and activate an SDK version",
            "description": "Install `sdk@version` (or `sdk@latest`) and make it the globally active version. Global shell environment files are updated; new shells pick up the change.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sdk": {"type": "string", "description": "SDK name, e.g. go, node"},
                    "version": {"type": "string", "description": "Version to install, or `latest`"},
                    "backend": {
                        "type": "string",
                        "enum": ["plugin", "pixi"],
                        "default": "plugin",
                        "description": "Install through the Lua plugin (default) or pixi/conda-forge"
                    }
                },
                "required": ["sdk", "version"],
                "additionalProperties": false
            },
            "annotations": {
                "destructiveHint": true,
                "openWorldHint": true,
                "idempotentHint": true
            }
        },
        {
            "name": "vmr_uninstall",
            "title": "Uninstall an SDK version",
            "description": "Uninstall one version of an SDK, or all versions when `version` is `all`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sdk": {"type": "string", "description": "SDK name"},
                    "version": {"type": "string", "description": "Version to remove, or `all`"}
                },
                "required": ["sdk", "version"],
                "additionalProperties": false
            },
            "annotations": {"destructiveHint": true, "idempotentHint": true}
        },
        {
            "name": "vmr_freeze",
            "title": "Export installed versions",
            "description": "Write installed SDK versions to a requirements-style file (`sdk@version` per line, `pixi` suffix for pixi installs). Replay it with `vmr use -f`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Output path; defaults to vmr_<YYYYMMDD>.txt in the server's working directory"}
                },
                "additionalProperties": false
            },
            "annotations": {"destructiveHint": false, "idempotentHint": true}
        },
        {
            "name": "vmr_env",
            "title": "Get SDK environment",
            "description": "Return PATH entries and environment variables for the active version of an SDK, so a client can run its binaries without modifying its own environment.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sdk": {"type": "string", "description": "SDK name"}
                },
                "required": ["sdk"],
                "additionalProperties": false
            },
            "annotations": {"readOnlyHint": true}
        }
    ])
}

fn call_tool(params: &Value) -> Result<Value, RpcError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::invalid_params("missing tool name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !args.is_object() {
        return Err(RpcError::invalid_params("`arguments` must be an object"));
    }
    match name {
        "vmr_list_plugins" => Ok(tool_result(
            crate::plugin_rows(false).map(installed_plugins_json),
        )),
        "vmr_search_versions" => {
            let sdk = str_arg(&args, "sdk")?;
            let backend = opt_str_arg(&args, "backend")?.unwrap_or_else(|| "plugin".to_string());
            let pixi = match backend.as_str() {
                "plugin" => false,
                "pixi" => true,
                _ => {
                    return Err(RpcError::invalid_params(
                        "`backend` must be `plugin` or `pixi`",
                    ));
                }
            };
            let no_cache = opt_bool_arg(&args, "no_cache")?.unwrap_or(false);
            Ok(tool_result(
                crate::search_versions(&sdk, pixi, no_cache)
                    .map(|rows| versions_json(&sdk, &backend, rows)),
            ))
        }
        "vmr_list_installed" => {
            let sdk = opt_str_arg(&args, "sdk")?;
            let entries = match sdk {
                Some(sdk) => vec![crate::sdk_versions(&sdk)],
                None => crate::installed_versions_all(),
            };
            Ok(tool_result(Ok(installed_json(entries))))
        }
        "vmr_install" => {
            let sdk = str_arg(&args, "sdk")?;
            let version = str_arg(&args, "version")?;
            let backend = opt_str_arg(&args, "backend")?.unwrap_or_else(|| "plugin".to_string());
            let pixi = match backend.as_str() {
                "plugin" => false,
                "pixi" => true,
                _ => {
                    return Err(RpcError::invalid_params(
                        "`backend` must be `plugin` or `pixi`",
                    ));
                }
            };
            let spec = format!("{sdk}@{version}");
            Ok(tool_result(
                crate::install_spec(&spec, pixi).map(install_json),
            ))
        }
        "vmr_uninstall" => {
            let sdk = str_arg(&args, "sdk")?;
            let version = str_arg(&args, "version")?;
            let spec = format!("{sdk}@{version}");
            Ok(tool_result(
                crate::uninstall_spec(&spec).map(|message| json!({"message": message})),
            ))
        }
        "vmr_freeze" => {
            let file = opt_str_arg(&args, "file")?;
            Ok(tool_result(crate::freeze_to(file.as_deref()).map(
                |(path, entries)| {
                    json!({
                        "file": path.display().to_string(),
                        "entries": entries
                    })
                },
            )))
        }
        "vmr_env" => {
            let sdk = str_arg(&args, "sdk")?;
            Ok(tool_result(env_json(&sdk)))
        }
        other => Err(RpcError::invalid_params(format!("Unknown tool: {other}"))),
    }
}

fn tool_result(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({
            "content": [{"type": "text", "text": value.to_string()}],
            "isError": false
        }),
        Err(message) => json!({
            "content": [{"type": "text", "text": format!("error: {message}")}],
            "isError": true
        }),
    }
}

fn str_arg(args: &Value, key: &str) -> Result<String, RpcError> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| RpcError::invalid_params(format!("missing or invalid `{key}` (string)")))
}

fn opt_str_arg(args: &Value, key: &str) -> Result<Option<String>, RpcError> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(RpcError::invalid_params(format!(
            "`{key}` must be a string"
        ))),
    }
}

fn opt_bool_arg(args: &Value, key: &str) -> Result<Option<bool>, RpcError> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(RpcError::invalid_params(format!(
            "`{key}` must be a boolean"
        ))),
    }
}

fn versions_json(sdk: &str, backend: &str, rows: Vec<VersionRow>) -> Value {
    json!({
        "sdk": sdk,
        "backend": backend,
        "versions": rows.into_iter().map(|r| json!({
            "version": r.version,
            "installer": r.installer,
            "os": r.os,
            "arch": r.arch
        })).collect::<Vec<_>>()
    })
}

fn installed_plugins_json(rows: Vec<PluginRow>) -> Value {
    json!({
        "plugins": rows.into_iter().map(|p| json!({
            "plugin": p.plugin_name,
            "sdk": p.sdk_name,
            "version": p.plugin_version,
            "homepage": p.homepage
        })).collect::<Vec<_>>()
    })
}

fn installed_json(entries: Vec<SdkVersions>) -> Value {
    json!({
        "sdks": entries.into_iter().map(|e| json!({
            "sdk": e.sdk,
            "plugin": e.label,
            "current": e.current,
            "versions": e.versions
        })).collect::<Vec<_>>()
    })
}

fn install_json(summary: InstallSummary) -> Value {
    json!({
        "sdk": summary.sdk_name,
        "plugin": summary.plugin_name,
        "version": summary.version,
        "backend": if summary.pixi { "pixi" } else { "plugin" },
        "activePath": summary.path.display().to_string(),
        "alreadyInstalled": summary.already_installed
    })
}

fn env_json(sdk: &str) -> Result<Value, String> {
    let Some(mut plugin) = crate::find_plugin_for(sdk) else {
        // Pixi-only installs have no plugin env metadata; the symlink itself is the PATH entry.
        let entry = crate::sdk_versions(sdk);
        let sym = vmr_installer::common::symbol_link_path(&entry.sdk);
        if !sym.exists() {
            return Err(format!("{} is not installed", entry.sdk));
        }
        return Ok(json!({
            "sdk": entry.sdk,
            "current": entry.current,
            "pathDirs": [sym.display().to_string()],
            "envVars": {}
        }));
    };
    let ic = plugin.get_installer_config()?;
    let sym = vmr_installer::common::symbol_link_path(&plugin.sdk_name);
    if !sym.exists() {
        return Err(format!("{} is not installed", plugin.sdk_name));
    }
    let os = vmr_pixi::platform::os_name();
    let bundle = vmr_env::collect(&ic, sym.to_string_lossy().as_ref(), &os);
    let mut vars = Map::new();
    for (key, value) in bundle.env_vars {
        vars.insert(key, Value::String(value));
    }
    Ok(json!({
        "sdk": plugin.sdk_name,
        "current": crate::sdk_versions(sdk).current,
        "pathDirs": bundle.path_dirs,
        "envVars": Value::Object(vars)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modern_request(method: &str, params: Value) -> Value {
        let mut params = match params {
            Value::Object(map) => map,
            _ => panic!("params must be an object"),
        };
        params.insert(
            "_meta".to_string(),
            json!({
                "io.modelcontextprotocol/protocolVersion": MODERN_VERSION,
                "io.modelcontextprotocol/clientCapabilities": {}
            }),
        );
        json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params})
    }

    #[test]
    fn legacy_initialize_negotiates_a_supported_version() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-06-18", "capabilities": {}}
        });
        let response = handle_message(&request).unwrap();
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(response["result"]["capabilities"]["tools"], json!({}));
        assert!(response["result"].get("resultType").is_none());
    }

    #[test]
    fn legacy_initialize_falls_back_to_latest_legacy_version() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "1900-01-01", "capabilities": {}}
        });
        let response = handle_message(&request).unwrap();
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
    }

    #[test]
    fn modern_tools_list_has_result_type_and_cache_fields() {
        let response = handle_message(&modern_request("tools/list", json!({}))).unwrap();
        let result = &response["result"];
        assert_eq!(result["resultType"], "complete");
        assert!(result["ttlMs"].is_number());
        assert_eq!(result["cacheScope"], "public");
        assert!(result["tools"].as_array().unwrap().len() >= 5);
        let names: Vec<&str> = result["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"vmr_install"));
        assert_eq!(names.first(), Some(&"vmr_list_plugins"));
    }

    #[test]
    fn modern_request_requires_client_capabilities() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {"_meta": {"io.modelcontextprotocol/protocolVersion": MODERN_VERSION}}
        });
        let response = handle_message(&request).unwrap();
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn modern_request_rejects_unsupported_version() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {"_meta": {
                "io.modelcontextprotocol/protocolVersion": "1900-01-01",
                "io.modelcontextprotocol/clientCapabilities": {}
            }}
        });
        let response = handle_message(&request).unwrap();
        assert_eq!(response["error"]["code"], -32022);
        assert_eq!(response["error"]["data"]["requested"], "1900-01-01");
    }

    #[test]
    fn server_discover_reports_versions_and_tools() {
        let response = handle_message(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "server/discover",
            "params": {}
        }))
        .unwrap();
        assert_eq!(
            response["result"]["supportedVersions"],
            json!([MODERN_VERSION])
        );
        assert_eq!(response["result"]["capabilities"]["tools"], json!({}));
        assert!(
            response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"].is_string()
        );
    }

    #[test]
    fn unknown_method_and_bad_tool_are_protocol_errors() {
        let unknown = handle_message(&json!({
            "jsonrpc": "2.0", "id": 5, "method": "nope", "params": {}
        }))
        .unwrap();
        assert_eq!(unknown["error"]["code"], -32601);

        let bad_tool = handle_message(&modern_request(
            "tools/call",
            json!({"name": "nope", "arguments": {}}),
        ))
        .unwrap();
        assert_eq!(bad_tool["error"]["code"], -32602);
    }

    #[test]
    fn missing_tool_arguments_are_invalid_params() {
        let response = handle_message(&modern_request(
            "tools/call",
            json!({"name": "vmr_install", "arguments": {"sdk": "go"}}),
        ))
        .unwrap();
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn notifications_get_no_response() {
        assert!(
            handle_message(&json!({
                "jsonrpc": "2.0", "method": "notifications/initialized"
            }))
            .is_none()
        );
    }
}
