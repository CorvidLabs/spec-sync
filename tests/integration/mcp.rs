#[cfg(windows)]
use crate::commands::create_windows_junction;
use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

#[path = "mcp/configuration.rs"]
mod configuration;

#[path = "mcp/gradle.rs"]
mod gradle;

#[path = "mcp/issues.rs"]
mod issues;

#[path = "mcp/limits.rs"]
mod limits;

#[path = "mcp/protocol.rs"]
mod protocol;

#[path = "mcp/read_roots.rs"]
mod read_roots;

#[path = "mcp/snapshots.rs"]
mod snapshots;

#[path = "mcp/tools.rs"]
mod tools;

#[path = "mcp/workspaces.rs"]
mod workspaces;

fn setup_minimal_mcp_project_at(root: &std::path::Path) {
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
}

fn coverage_request(id: u64, root: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "specsync_coverage",
            "arguments": { "root": root }
        }
    })
}

fn mcp_request_with_write(
    root: &std::path::Path,
    requests: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    let input = requests
        .iter()
        .map(|request| serde_json::to_string(request).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let output = specsync()
        .arg("mcp")
        .arg("--allow-write")
        .arg("--root")
        .arg(root)
        .write_stdin(input)
        .output()
        .expect("failed to run write-enabled mcp");
    assert!(
        output.status.success(),
        "write-enabled MCP process failed with status {}; stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("invalid JSON-RPC response"))
        .collect()
}
