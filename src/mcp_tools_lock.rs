//! MCP tools lock — fail-closed fingerprint of the MCP tool catalog.
//!
//! SoT path: `.specsync/mcp-tools.lock.json`
//! Canonical hash: SHA-256 of UTF-8 JSON with sorted object keys and
//! separators `(',', ':')` over `{name, title?, description, inputSchema}`.
//! `title` is omitted when null/absent.

use crate::mcp;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const LOCK_RELATIVE_PATH: &str = ".specsync/mcp-tools.lock.json";
const LOCK_FORMAT_VERSION: u32 = 1;

/// Build the lock document from the same catalog the MCP server registers.
fn build_lock(allow_write: bool) -> Value {
    let tools = mcp::mcp_tool_definitions(allow_write);
    let lock_tools: Vec<Value> = tools.iter().map(lock_entry_from_tool).collect();
    json!({
        "version": LOCK_FORMAT_VERSION,
        "generated_by": format!("specsync {}", env!("CARGO_PKG_VERSION")),
        "server": if allow_write { "specsync mcp --allow-write" } else { "specsync mcp" },
        "tool_count": lock_tools.len(),
        "tools": lock_tools,
    })
}

/// Print lock JSON to stdout (pretty).
pub fn cmd_lock_print(allow_write: bool) -> Result<(), String> {
    let lock = build_lock(allow_write);
    let text = serde_json::to_string_pretty(&lock)
        .map_err(|e| format!("Failed to serialize MCP tools lock: {e}"))?;
    println!("{text}");
    Ok(())
}

/// Explicitly write `.specsync/mcp-tools.lock.json` under `root`.
pub fn cmd_lock_write(root: &Path, allow_write: bool) -> Result<(), String> {
    let lock = build_lock(allow_write);
    let path = lock_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&lock)
        .map_err(|e| format!("Failed to serialize MCP tools lock: {e}"))?;
    let mut file =
        fs::File::create(&path).map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

/// Compare live catalog to committed lock. Exit code semantics via Result:
/// Ok(()) = match (exit 0); Err(message) = drift/missing (caller exits non-zero).
pub fn cmd_diff(root: &Path, allow_write: bool) -> Result<(), String> {
    let path = lock_path(root);
    if !path.exists() {
        return Err(format!(
            "MCP tools lock missing: {}\n\
             Remedy: run `specsync mcp lock --write` and commit the file.",
            path.display()
        ));
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let locked: Value = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Invalid MCP tools lock JSON at {}: {e}\n\
             Remedy: regenerate with `specsync mcp lock --write`.",
            path.display()
        )
    })?;
    diff_live_against_lock(&locked, allow_write)
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(LOCK_RELATIVE_PATH)
}

fn lock_entry_from_tool(tool: &Value) -> Value {
    let name = tool
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let description = tool
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let input_schema = tool
        .get("inputSchema")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let title = tool.get("title").cloned();
    let sha = tool_contract_sha256(
        name.as_str(),
        title.as_ref(),
        description.as_str(),
        &input_schema,
    );
    let mut entry = Map::new();
    entry.insert("name".into(), Value::String(name));
    if let Some(title) = title {
        if !title.is_null() {
            entry.insert("title".into(), title);
        }
    }
    entry.insert("description".into(), Value::String(description));
    entry.insert("inputSchema".into(), input_schema);
    entry.insert("sha256".into(), Value::String(sha));
    Value::Object(entry)
}

/// Canonical SHA-256 over controlled fields. Omits `title` when null/absent.
fn tool_contract_sha256(
    name: &str,
    title: Option<&Value>,
    description: &str,
    input_schema: &Value,
) -> String {
    let mut controlled = Map::new();
    controlled.insert("name".into(), Value::String(name.to_string()));
    if let Some(title) = title {
        if !title.is_null() {
            controlled.insert("title".into(), title.clone());
        }
    }
    controlled.insert("description".into(), Value::String(description.to_string()));
    controlled.insert("inputSchema".into(), input_schema.clone());
    let canonical = canonical_json(&Value::Object(controlled));
    hex_sha256(canonical.as_bytes())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// JSON with recursively sorted object keys and compact separators.
fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            // serde_json's escape rules match RFC 8259 / Python json.dumps
            out.push_str(&serde_json::to_string(s).expect("string serialization"));
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let keys: BTreeMap<&str, &Value> = map.iter().map(|(k, v)| (k.as_str(), v)).collect();
            out.push('{');
            for (i, (k, v)) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).expect("key serialization"));
                out.push(':');
                write_canonical(v, out);
            }
            out.push('}');
        }
    }
}

fn diff_live_against_lock(locked: &Value, allow_write: bool) -> Result<(), String> {
    let live = build_lock(allow_write);
    let mut problems: Vec<String> = Vec::new();

    let locked_count = locked
        .get("tool_count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let live_count = live.get("tool_count").and_then(Value::as_u64).unwrap_or(0) as usize;
    if locked_count != live_count {
        problems.push(format!(
            "tool_count mismatch: lock={locked_count} live={live_count}"
        ));
    }

    let locked_tools = index_tools(locked.get("tools").unwrap_or(&Value::Null));
    let live_tools = index_tools(live.get("tools").unwrap_or(&Value::Null));

    let locked_names: BTreeSet<&str> = locked_tools.keys().copied().collect();
    let live_names: BTreeSet<&str> = live_tools.keys().copied().collect();

    for name in live_names.difference(&locked_names) {
        problems.push(format!("tool added (not in lock): {name}"));
    }
    for name in locked_names.difference(&live_names) {
        problems.push(format!(
            "tool removed or renamed (in lock, not live): {name}"
        ));
    }
    for name in locked_names.intersection(&live_names) {
        let locked_sha = locked_tools[name]
            .get("sha256")
            .and_then(Value::as_str)
            .unwrap_or("");
        let live_sha = live_tools[name]
            .get("sha256")
            .and_then(Value::as_str)
            .unwrap_or("");
        if locked_sha != live_sha {
            problems.push(format!(
                "sha256 mismatch for `{name}`: lock={locked_sha} live={live_sha}"
            ));
        }
    }

    if problems.is_empty() {
        println!("MCP tools lock matches live catalog ({live_count} tools).");
        Ok(())
    } else {
        let mut msg = String::from("MCP tools lock drift detected:\n");
        for p in &problems {
            msg.push_str("  - ");
            msg.push_str(p);
            msg.push('\n');
        }
        msg.push_str(
            "Remedy: review the intentional surface change, then run \
             `specsync mcp lock --write` and commit `.specsync/mcp-tools.lock.json`.",
        );
        Err(msg)
    }
}

fn index_tools(tools: &Value) -> BTreeMap<&str, &Value> {
    let mut map = BTreeMap::new();
    if let Some(arr) = tools.as_array() {
        for tool in arr {
            if let Some(name) = tool.get("name").and_then(Value::as_str) {
                map.insert(name, tool);
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn canonical_json_sorts_keys_and_is_compact() {
        let v = json!({"b": 1, "a": {"z": true, "y": [2, 1]}});
        assert_eq!(canonical_json(&v), r#"{"a":{"y":[2,1],"z":true},"b":1}"#);
    }

    #[test]
    fn sha256_omits_null_title() {
        let schema = json!({"type": "object"});
        let with_null = tool_contract_sha256("t", Some(&Value::Null), "d", &schema);
        let without = tool_contract_sha256("t", None, "d", &schema);
        assert_eq!(with_null, without);
        let with_title = tool_contract_sha256("t", Some(&json!("Title")), "d", &schema);
        assert_ne!(with_null, with_title);
    }

    #[test]
    fn match_returns_ok() {
        let dir = tempdir().unwrap();
        cmd_lock_write(dir.path(), false).unwrap();
        assert!(cmd_diff(dir.path(), false).is_ok());
    }

    #[test]
    fn missing_lock_returns_err_with_remedy() {
        let dir = tempdir().unwrap();
        let err = cmd_diff(dir.path(), false).unwrap_err();
        assert!(err.contains("missing"), "{err}");
        assert!(err.contains("mcp lock --write"), "{err}");
    }

    #[test]
    fn description_drift_returns_nonzero() {
        let dir = tempdir().unwrap();
        cmd_lock_write(dir.path(), false).unwrap();
        let path = lock_path(dir.path());
        let mut lock: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let tools = lock.get_mut("tools").unwrap().as_array_mut().unwrap();
        let desc = tools[0].get_mut("description").unwrap();
        *desc = json!("intentionally drifted description");
        // Also corrupt sha so mismatch is detected via sha path
        tools[0]
            .as_object_mut()
            .unwrap()
            .insert("sha256".into(), json!("0".repeat(64)));
        fs::write(&path, serde_json::to_string_pretty(&lock).unwrap()).unwrap();
        let err = cmd_diff(dir.path(), false).unwrap_err();
        assert!(err.contains("sha256 mismatch"), "{err}");
    }

    #[test]
    fn schema_drift_via_sha_mismatch() {
        let dir = tempdir().unwrap();
        let mut lock = build_lock(false);
        {
            let tools = lock.get_mut("tools").unwrap().as_array_mut().unwrap();
            let tool = tools[0].as_object_mut().unwrap();
            tool.get_mut("inputSchema")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("extra".into(), json!(true));
            // Recompute sha so the lock claims a different schema contract.
            let name = tool
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .to_string();
            let description = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap()
                .to_string();
            let schema = tool.get("inputSchema").cloned().unwrap();
            let sha = tool_contract_sha256(&name, None, &description, &schema);
            tool.insert("sha256".into(), json!(sha));
        }
        let path = lock_path(dir.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&lock).unwrap()).unwrap();
        let err = cmd_diff(dir.path(), false).unwrap_err();
        assert!(err.contains("sha256 mismatch"), "{err}");
    }

    #[test]
    fn removed_tool_detected() {
        let dir = tempdir().unwrap();
        let mut lock = build_lock(false);
        let new_count = {
            let tools = lock.get_mut("tools").unwrap().as_array_mut().unwrap();
            tools.pop();
            tools.len()
        };
        lock.as_object_mut()
            .unwrap()
            .insert("tool_count".into(), json!(new_count));
        let path = lock_path(dir.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&lock).unwrap()).unwrap();
        let err = cmd_diff(dir.path(), false).unwrap_err();
        assert!(
            err.contains("tool added") || err.contains("tool_count"),
            "{err}"
        );
    }

    #[test]
    fn lock_uses_same_catalog_as_server() {
        let lock = build_lock(false);
        let defs = mcp::mcp_tool_definitions(false);
        assert_eq!(
            lock.get("tool_count").and_then(Value::as_u64).unwrap() as usize,
            defs.len()
        );
        let lock_names: Vec<_> = lock["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        let def_names: Vec<_> = defs.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(lock_names, def_names);
    }
}
