use super::*;

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
        "include ':outside'\nproject(':outside').projectDir = file('../outside-gradle')\n",
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
    assert_eq!(
        cycle_responses[0]["result"]["isError"], true,
        "unexpected cycle response: {}",
        cycle_responses[0]
    );
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
            .contains("input paths")
    );
}

#[test]
fn mcp_manifest_traversal_accepts_duplicate_cargo_and_node_workspaces_once() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_minimal_mcp_project_at(root);
    fs::create_dir_all(root.join("crates/member/src")).unwrap();
    fs::create_dir_all(root.join("packages/member")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("packages/member/package.json"),
        r#"{"name":"member"}"#,
    )
    .unwrap();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_ne!(
        responses[0]["result"]["isError"], true,
        "duplicate workspace declarations must not trigger replay or a false failure: {}",
        responses[0]
    );
}

#[test]
fn mcp_package_json_failures_are_inconclusive_for_tools_and_resources() {
    for (label, content, expected) in [
        ("malformed", "{", "Cannot parse MCP package.json as JSON"),
        ("non-object", "[]", "root must be a JSON object"),
        (
            "wrong-workspaces",
            r#"{"workspaces":7}"#,
            "`workspaces` must be an array or an object",
        ),
        (
            "wrong-packages",
            r#"{"workspaces":{"packages":"packages/*"}}"#,
            "`workspaces.packages` must be an array",
        ),
        (
            "missing-packages",
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
        (
            "wrong-entry",
            r#"{"workspaces":["packages/*",false]}"#,
            "workspace entries must be strings",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        setup_minimal_mcp_project_at(root);
        fs::write(root.join("package.json"), content).unwrap();

        let responses = mcp_request(
            root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_coverage", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///coverage" }
                }),
            ],
        );

        assert_eq!(responses.len(), 2, "{label}: {responses:#?}");
        assert_eq!(
            responses[0]["result"]["isError"], true,
            "{label}: {responses:#?}"
        );
        assert_eq!(
            responses[1]["error"]["code"], -32602,
            "{label}: {responses:#?}"
        );
        let rendered = serde_json::to_string(&responses).unwrap();
        assert!(rendered.contains(expected), "{label}: {rendered}");
    }
}

#[test]
fn mcp_nested_package_json_failures_are_inconclusive_for_tools_and_resources() {
    for (label, content, expected) in [
        (
            "malformed",
            "{",
            "Cannot parse MCP package workspace manifest",
        ),
        ("non-object", "[]", "root must be a JSON object"),
        (
            "missing-packages",
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
        (
            "wrong-packages",
            r#"{"workspaces":{"packages":false}}"#,
            "`workspaces.packages` must be an array",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        setup_minimal_mcp_project_at(root);
        fs::create_dir_all(root.join("packages/member")).unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(root.join("packages/member/package.json"), content).unwrap();

        let responses = mcp_request(
            root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_coverage", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///coverage" }
                }),
            ],
        );

        for response in responses {
            let encoded = response.to_string();
            assert!(
                encoded.contains(expected),
                "{label}: expected {expected:?} in {encoded}"
            );
            assert!(
                response["result"]["isError"] == true || response["error"].is_object(),
                "{label}: malformed nested package manifest must not report success: {encoded}"
            );
        }
    }
}

#[test]
fn mcp_multiline_cargo_workspace_members_survive_coverage_and_generation() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("vendor/member/src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"vendor/member\",\n]\n",
    )
    .unwrap();
    fs::write(
        root.join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("vendor/member/src/lib.rs"),
        "pub fn workspace_member() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/member")).unwrap();
    fs::write(
        root.join("specs/member/member.spec.md"),
        valid_spec("member", &[]),
    )
    .unwrap();

    let responses = mcp_request_with_write(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": { "name": "specsync_generate", "arguments": {} }
            }),
        ],
    );

    assert_eq!(responses.len(), 2, "{responses:#?}");
    assert_ne!(
        responses[0]["result"]["isError"], true,
        "multiline Cargo member vanished from coverage: {responses:#?}"
    );
    let coverage: serde_json::Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(coverage["files_total"], 1, "{coverage}");
    assert_eq!(
        coverage["uncovered_files"][0]["file"], "vendor/member/src/lib.rs",
        "{coverage}"
    );

    assert_ne!(
        responses[1]["result"]["isError"], true,
        "multiline Cargo member vanished from generation: {responses:#?}"
    );
    let generation: serde_json::Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(
        generation["count"].as_u64().is_some_and(|count| count >= 1),
        "{generation}"
    );
    assert!(
        generation["generated"]
            .as_array()
            .is_some_and(|paths| paths.iter().any(|path| path
                .as_str()
                .is_some_and(|path| path.ends_with("src.spec.md")))),
        "{generation}"
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
