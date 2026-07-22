use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

// ─── MCP Server Tests ──────────────────────────────────────────────────────

#[test]
fn mcp_initialize_returns_capabilities() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })],
    );

    assert_eq!(responses.len(), 1);
    let result = &responses[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "specsync");
    assert!(result["capabilities"]["tools"].is_object());
}

#[test]
fn mcp_tools_list_defaults_to_read_only_tools() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        })],
    );

    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"specsync_check"));
    assert!(tool_names.contains(&"specsync_coverage"));
    assert!(tool_names.contains(&"specsync_list_specs"));
    assert!(tool_names.contains(&"specsync_score"));
    assert!(tool_names.contains(&"specsync_issues"));
    assert!(!tool_names.contains(&"specsync_generate"));
    assert!(!tool_names.contains(&"specsync_init"));
    assert_eq!(tools.len(), 5);
    assert!(
        tools
            .iter()
            .all(|tool| tool["inputSchema"]["additionalProperties"] == false)
    );
}

#[test]
fn mcp_allow_write_lists_mutating_tools_with_exact_schemas() {
    let tmp = TempDir::new().unwrap();
    let responses = mcp_request_with_write(
        tmp.path(),
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        })],
    );

    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(tools.len(), 7);
    assert!(tool_names.contains(&"specsync_generate"));
    assert!(tool_names.contains(&"specsync_init"));
    for tool in tools {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    }
}

#[test]
fn mcp_generate_rejects_retired_ai_arguments_without_echoing_credentials() {
    let tmp = TempDir::new().unwrap();
    let secret = "sk-never-echo-this";
    let responses = mcp_request_with_write(
        tmp.path(),
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_generate",
                "arguments": { "apiKey": secret }
            }
        })],
    );

    assert_eq!(responses[0]["error"]["code"], -32602);
    let message = responses[0]["error"]["message"].as_str().unwrap();
    assert!(message.contains("removed in spec-sync 5.0"));
    assert!(!message.contains(secret));
}

#[test]
fn mcp_tool_check_validates_specs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_check",
                "arguments": {}
            }
        })],
    );

    assert_eq!(responses.len(), 1);
    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["passed"].as_bool().unwrap());
    assert_eq!(result["specs_checked"].as_u64().unwrap(), 1);
}

#[test]
fn mcp_tool_coverage_returns_metrics() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_coverage",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["files_total"].as_u64().unwrap() > 0);
    assert!(result["file_coverage"].is_number());
}

#[test]
fn mcp_tool_init_creates_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    let responses = mcp_request_with_write(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_init",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["created"].as_bool().unwrap());
    assert!(root.join("specsync.json").exists());
}

#[test]
fn mcp_read_only_rejects_direct_mutators_and_preserves_outside_victim() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"outside victim bytes\0must stay exact";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request(
        &root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": {}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": {}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
        ],
    );

    assert_eq!(responses.len(), 4);
    for response in &responses[..2] {
        assert_eq!(response["result"]["isError"], true);
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("--allow-write")
        );
    }
    assert_eq!(responses[2]["error"]["code"], -32602);
    assert_eq!(responses[3]["error"]["code"], -32602);
    assert!(!root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert!(!outside.join("specs").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_allow_write_uses_server_root_and_rejects_root_overrides() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"write override victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request_with_write(
        &root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_init", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
        ],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    assert_eq!(responses[1]["error"]["code"], -32602);
    assert_eq!(responses[2]["error"]["code"], -32602);
    assert!(root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert!(!outside.join("specs").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_read_roots_allow_existing_children_and_reject_escapes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let child = root.join("child");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(child.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"read confinement victim";
    fs::write(&victim, victim_bytes).unwrap();
    let nonexistent_outside = outside.join("missing");

    let responses = mcp_request(
        &root,
        &[
            coverage_request(1, serde_json::json!("child")),
            coverage_request(2, serde_json::json!(outside.to_string_lossy())),
            coverage_request(3, serde_json::json!(nonexistent_outside.to_string_lossy())),
            coverage_request(4, serde_json::json!("../outside")),
        ],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    for response in &responses[1..] {
        assert_eq!(response["result"]["isError"], true);
    }
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(unix)]
#[test]
fn mcp_read_root_rejects_symlink_escape_and_preserves_referent() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("escape")).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"symlink escape victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request(&root, &[coverage_request(1, serde_json::json!("escape"))]);

    assert_eq!(responses[0]["result"]["isError"], true);
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_rejects_configured_read_and_write_path_escapes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(outside.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "pub fn local() {}\n").unwrap();
    fs::write(outside.join("src/secret.rs"), "pub fn secret() {}\n").unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"configured path victim";
    fs::write(&victim, victim_bytes).unwrap();

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"../outside/specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let write_responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(write_responses[0]["result"]["isError"], true);
    assert!(
        write_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("specs_dir")
    );
    assert!(!outside.join("specs").exists());

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["../outside/src"]}"#,
    )
    .unwrap();
    let read_responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(read_responses[0]["result"]["isError"], true);
    assert!(
        read_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("source_dirs")
    );
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(unix)]
#[test]
fn mcp_rejects_configured_symlink_trees_and_dangling_write_destinations() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"configured symlink victim";
    fs::write(&victim, victim_bytes).unwrap();

    let read_root = tmp.path().join("read-server");
    fs::create_dir_all(read_root.join("specs")).unwrap();
    fs::create_dir_all(read_root.join("src/nested")).unwrap();
    symlink(&victim, read_root.join("src/nested/escape.rs")).unwrap();
    fs::write(
        read_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let read_responses = mcp_request(
        &read_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(read_responses[0]["result"]["isError"], true);

    let write_root = tmp.path().join("write-server");
    fs::create_dir_all(write_root.join("src")).unwrap();
    fs::create_dir_all(write_root.join("specs")).unwrap();
    symlink(&outside, write_root.join("specs/evil")).unwrap();
    fs::write(write_root.join("src/main.rs"), "pub fn main() {}\n").unwrap();
    fs::write(
        write_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"modules":{"evil":{"files":["src/main.rs"]}}}"#,
    )
    .unwrap();
    let write_responses = mcp_request_with_write(
        &write_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(write_responses[0]["result"]["isError"], true);

    let init_root = tmp.path().join("init-server");
    fs::create_dir_all(&init_root).unwrap();
    let outside_config = outside.join("new-config.json");
    symlink(&outside_config, init_root.join("specsync.json")).unwrap();
    let init_responses = mcp_request_with_write(
        &init_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        })],
    );
    assert_eq!(init_responses[0]["result"]["isError"], true);
    assert!(!outside_config.exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_rejects_unsafe_module_names_and_spec_file_mappings() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/local")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/main.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("victim.rs");
    let victim_bytes = b"pub fn outside_secret() {}\n";
    fs::write(&victim, victim_bytes).unwrap();

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"modules":{"bad\\name":{"files":["src/main.rs"]}}}"#,
    )
    .unwrap();
    let module_responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(module_responses[0]["result"]["isError"], true);
    assert!(
        module_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("module name")
    );

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1.0.0\nstatus: stable\nfiles:\n  - ../outside/victim.rs\n---\n\n# Local\n",
    )
    .unwrap();
    let mapping_responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );
    assert_eq!(mapping_responses[0]["result"]["isError"], true);
    assert!(
        mapping_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("spec file mapping")
    );
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(unix)]
#[test]
fn mcp_rejects_metadata_symlinks_and_traversing_dependency_references() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let outside_manifest = outside.join("package.json");
    fs::write(&outside_manifest, r#"{"name":"outside"}"#).unwrap();

    let manifest_root = tmp.path().join("manifest-server");
    fs::create_dir_all(manifest_root.join("src")).unwrap();
    symlink(&outside_manifest, manifest_root.join("package.json")).unwrap();
    let manifest_responses = mcp_request(
        &manifest_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(manifest_responses[0]["result"]["isError"], true);

    let cache_root = tmp.path().join("cache-server");
    fs::create_dir_all(cache_root.join("src")).unwrap();
    fs::create_dir_all(cache_root.join("specs/local")).unwrap();
    fs::create_dir_all(cache_root.join(".specsync")).unwrap();
    fs::write(cache_root.join("src/local.rs"), "pub fn local() {}\n").unwrap();
    fs::write(
        cache_root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1\nstatus: stable\nfiles:\n  - src/local.rs\n---\n\n# Local\n",
    )
    .unwrap();
    symlink(&outside_manifest, cache_root.join(".specsync/hashes.json")).unwrap();
    let cache_responses = mcp_request(
        &cache_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_check", "arguments": {} }
        })],
    );
    assert_eq!(cache_responses[0]["result"]["isError"], true);

    fs::remove_file(cache_root.join(".specsync/hashes.json")).unwrap();
    fs::write(
        cache_root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1\nstatus: stable\nfiles:\n  - src/local.rs\ndepends_on:\n  - ../../outside/package.json\n---\n\n# Local\n\n### Consumed By\n\n| Module | Purpose |\n|---|---|\n| `../../outside/package.json` | outside |\n",
    )
    .unwrap();
    let dependency_responses = mcp_request(
        &cache_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );
    assert_eq!(dependency_responses[0]["result"]["isError"], true);
    assert!(
        dependency_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("dependency reference")
    );
}

#[cfg(unix)]
#[test]
fn mcp_confinement_scan_honors_configured_excluded_directories() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(root.join("ignored")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("main.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("ignored.rs");
    fs::write(&victim, "pub fn ignored() {}\n").unwrap();
    symlink(&victim, root.join("ignored/escape.rs")).unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["."],"excludeDirs":["ignored"]}"#,
    )
    .unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
}

#[cfg(unix)]
#[test]
fn mcp_manifest_autodetection_rejects_workspace_escapes() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside_cargo = tmp.path().join("outside-cargo");
    let outside_packages = tmp.path().join("outside-packages/pkg");
    fs::create_dir_all(outside_cargo.join("src")).unwrap();
    fs::create_dir_all(outside_packages.join("src")).unwrap();
    let cargo_manifest_bytes = b"[package]\nname = \"outside-cargo\"\nversion = \"0.1.0\"\n";
    fs::write(outside_cargo.join("Cargo.toml"), cargo_manifest_bytes).unwrap();
    let package_manifest_bytes = br#"{"name":"outside-package"}"#;
    fs::write(
        outside_packages.join("package.json"),
        package_manifest_bytes,
    )
    .unwrap();

    let cargo_root = tmp.path().join("cargo-server");
    fs::create_dir_all(cargo_root.join("src")).unwrap();
    fs::write(cargo_root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
    symlink(&outside_cargo, cargo_root.join("linked-member")).unwrap();
    fs::write(
        cargo_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"../outside-cargo\", \"linked-member\"]\n",
    )
    .unwrap();
    fs::write(
        cargo_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let cargo_responses = mcp_request(
        &cargo_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(cargo_responses[0]["result"]["isError"], true);
    fs::remove_file(cargo_root.join("specsync.json")).unwrap();
    let cargo_init_responses = mcp_request_with_write(
        &cargo_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        })],
    );
    assert_eq!(cargo_init_responses[0]["result"]["isError"], true);

    let package_root = tmp.path().join("package-server");
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        package_root.join("src/index.ts"),
        "export const local = 1;\n",
    )
    .unwrap();
    fs::write(
        package_root.join("package.json"),
        r#"{"name":"local","workspaces":["../outside-packages/*"]}"#,
    )
    .unwrap();
    fs::write(
        package_root.join("specsync.json"),
        r#"{"specsDir":"specs"}"#,
    )
    .unwrap();
    let package_responses = mcp_request(
        &package_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(package_responses[0]["result"]["isError"], true);

    let linked_package_root = tmp.path().join("linked-package-server");
    fs::create_dir_all(linked_package_root.join("src")).unwrap();
    fs::create_dir_all(linked_package_root.join("packages")).unwrap();
    fs::write(
        linked_package_root.join("src/index.ts"),
        "export const local = 1;\n",
    )
    .unwrap();
    symlink(
        &outside_packages,
        linked_package_root.join("packages/linked"),
    )
    .unwrap();
    fs::write(
        linked_package_root.join("package.json"),
        r#"{"name":"local","workspaces":["packages/*"]}"#,
    )
    .unwrap();
    fs::write(
        linked_package_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let linked_package_responses = mcp_request(
        &linked_package_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(linked_package_responses[0]["result"]["isError"], true);

    let scan_root = tmp.path().join("scan-server");
    fs::create_dir_all(&scan_root).unwrap();
    symlink(&outside_cargo, scan_root.join("linked-source")).unwrap();
    let scan_responses = mcp_request(
        &scan_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(scan_responses[0]["result"]["isError"], true);

    assert_eq!(
        fs::read(outside_cargo.join("Cargo.toml")).unwrap(),
        cargo_manifest_bytes
    );
    assert_eq!(
        fs::read(outside_packages.join("package.json")).unwrap(),
        package_manifest_bytes
    );
}

#[test]
fn mcp_manifest_autodetection_rejects_gradle_and_python_path_escapes() {
    let tmp = TempDir::new().unwrap();

    let outside_gradle = tmp.path().join("outside-gradle");
    fs::create_dir_all(outside_gradle.join("src/main/kotlin")).unwrap();
    let gradle_root = tmp.path().join("gradle-server");
    fs::create_dir_all(&gradle_root).unwrap();
    fs::write(gradle_root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        gradle_root.join("settings.gradle.kts"),
        "include(\":../outside-gradle\")\n",
    )
    .unwrap();
    fs::write(
        gradle_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let gradle_responses = mcp_request(
        &gradle_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(gradle_responses[0]["result"]["isError"], true);

    let outside_python = tmp.path().join("outside-python");
    fs::create_dir_all(&outside_python).unwrap();
    let python_root = tmp.path().join("python-server");
    fs::create_dir_all(&python_root).unwrap();
    fs::write(
        python_root.join("pyproject.toml"),
        "[project]\nname = \"../outside-python\"\n",
    )
    .unwrap();
    fs::write(python_root.join("specsync.json"), r#"{"specsDir":"specs"}"#).unwrap();
    let python_responses = mcp_request(
        &python_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(python_responses[0]["result"]["isError"], true);
}

#[test]
fn mcp_manifest_preflight_rejects_cycles_and_excessive_configured_paths() {
    let tmp = TempDir::new().unwrap();

    let cycle_root = tmp.path().join("cycle-server");
    fs::create_dir_all(&cycle_root).unwrap();
    fs::write(
        cycle_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\".\"]\n",
    )
    .unwrap();
    let cycle_responses = mcp_request(
        &cycle_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(cycle_responses[0]["result"]["isError"], true);
    assert!(
        cycle_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("cycle")
    );

    let bounded_root = tmp.path().join("bounded-server");
    fs::create_dir_all(&bounded_root).unwrap();
    let source_dirs: Vec<String> = (0..1_001).map(|index| format!("src-{index}")).collect();
    fs::write(
        bounded_root.join("specsync.json"),
        serde_json::to_vec(&serde_json::json!({
            "specsDir": "specs",
            "sourceDirs": source_dirs
        }))
        .unwrap(),
    )
    .unwrap();
    let bounded_responses = mcp_request(
        &bounded_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(bounded_responses[0]["result"]["isError"], true);
    assert!(
        bounded_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("path entries")
    );
}

#[cfg(unix)]
#[test]
fn mcp_autodetection_preflight_honors_builtin_ignored_directories() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/local.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("secret.rs");
    let victim_bytes = b"pub fn outside_secret() {}\n";
    fs::write(&victim, victim_bytes).unwrap();
    symlink(&victim, root.join("target/escape.rs")).unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_tools_call_rejects_shape_type_and_unknown_key_errors() {
    let tmp = TempDir::new().unwrap();
    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": []
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {}, "extra": true }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": 42, "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": [] }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 5, "method": "tools/call",
            "params": { "name": "specsync_check", "arguments": { "strict": "yes" } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 6, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": { "root": false } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": { "name": "specsync_score", "arguments": { "unexpected": true } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 8, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": { "root": "." } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {}
        }),
    ];

    let responses = mcp_request_with_write(tmp.path(), &requests);

    assert_eq!(responses.len(), requests.len());
    for response in responses {
        assert_eq!(response["error"]["code"], -32602);
        assert!(response.get("result").is_none());
    }
    assert!(!tmp.path().join("specsync.json").exists());
}

#[test]
fn mcp_notifications_never_respond_or_mutate() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"notification victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request_with_write(
        &root,
        &[
            serde_json::json!({ "jsonrpc": "2.0", "method": "initialize", "params": {} }),
            serde_json::json!({ "jsonrpc": "2.0", "method": "tools/list", "params": {} }),
            serde_json::json!({ "jsonrpc": "2.0", "method": "unknown/method" }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({ "jsonrpc": "2.0", "id": 99, "method": "ping" }),
        ],
    );

    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 99);
    assert!(!root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_parse_error_still_returns_json_rpc_error() {
    let tmp = TempDir::new().unwrap();
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(tmp.path())
        .write_stdin("{not-json\n")
        .output()
        .expect("failed to run mcp");
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(response["id"], serde_json::Value::Null);
    assert_eq!(response["error"]["code"], -32700);
}

#[test]
fn mcp_rejects_an_oversized_line_and_processes_the_next_request() {
    let tmp = TempDir::new().unwrap();
    let oversized = "x".repeat(1024 * 1024 + 1);
    let input = format!("{oversized}\n{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}}\n");
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(tmp.path())
        .write_stdin(input)
        .output()
        .expect("failed to run mcp");
    let responses: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], serde_json::Value::Null);
    assert_eq!(responses[0]["error"]["code"], -32700);
    assert!(
        responses[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("1 MiB")
    );
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[1]["result"], serde_json::json!({}));
}

#[test]
fn mcp_rejects_a_nonexistent_server_root() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("missing");
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(&missing)
        .write_stdin("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n")
        .output()
        .expect("failed to run mcp with missing root");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not exist or is not a directory")
    );
    assert!(!missing.exists());
}

#[test]
fn mcp_help_documents_explicit_write_authorization() {
    let output = specsync()
        .arg("mcp")
        .arg("--help")
        .output()
        .expect("failed to read mcp help");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("--allow-write"));
    assert!(stdout.contains("configured project root"));
}

#[test]
fn mcp_tool_list_specs_returns_spec_info() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_list_specs",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["count"].as_u64().unwrap() >= 1);
    let specs = result["specs"].as_array().unwrap();
    assert!(specs[0]["module"].is_string());
    assert!(specs[0]["path"].is_string());
}

#[test]
fn mcp_unknown_tool_returns_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        })],
    );

    let result = &responses[0]["result"];
    assert!(result["isError"].as_bool().unwrap());
}

#[test]
fn mcp_ping_returns_empty_result() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        })],
    );

    assert_eq!(responses.len(), 1);
    assert!(responses[0]["result"].is_object());
}

// ─── MCP Score Tool Tests ────────────────────────────────────────────────

#[test]
fn mcp_score_tool_returns_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "capabilities": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_score",
                    "arguments": {}
                }
            }),
        ],
    );

    assert_eq!(responses.len(), 2);
    let score_result = &responses[1]["result"]["content"][0]["text"];
    let score_json: serde_json::Value =
        serde_json::from_str(score_result.as_str().unwrap()).unwrap();
    assert!(score_json["average_score"].is_number());
    assert!(score_json["grade"].is_string());
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

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("invalid JSON-RPC response"))
        .collect()
}
