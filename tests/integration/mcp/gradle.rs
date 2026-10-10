use super::*;

#[cfg(unix)]
#[test]
fn mcp_tools_and_resources_reject_unsafe_gradle_build_manifests() {
    use std::os::unix::fs::symlink;
    use std::process::Command;

    for case in ["symlink", "fifo", "oversized"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        match case {
            "symlink" => {
                let outside = tmp.path().join("outside-build.gradle.kts");
                fs::write(&outside, "plugins { id(\"GRADLE_BUILD_SECRET\") }\n").unwrap();
                symlink(&outside, root.join("build.gradle.kts")).unwrap();
            }
            "fifo" => {
                assert!(
                    Command::new("mkfifo")
                        .arg(root.join("build.gradle.kts"))
                        .status()
                        .unwrap()
                        .success()
                );
            }
            "oversized" => {
                fs::write(
                    root.join("build.gradle.kts"),
                    vec![b' '; 4 * 1024 * 1024 + 1],
                )
                .unwrap();
            }
            _ => unreachable!(),
        }

        let responses = mcp_request(
            &root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_list_specs", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///config" }
                }),
            ],
        );

        assert_eq!(responses[0]["result"]["isError"], true, "{case}");
        assert_eq!(responses[1]["error"]["code"], -32602, "{case}");
        let rendered = serde_json::to_string(&responses).unwrap();
        assert!(rendered.contains("manifest"), "{case}: {rendered}");
        assert!(!rendered.contains("GRADLE_BUILD_SECRET"));
        if case == "oversized" {
            assert!(rendered.contains("4 MiB"), "{rendered}");
        } else {
            assert!(rendered.contains("regular file"), "{rendered}");
        }
    }
}

#[test]
fn mcp_gradle_set_project_dir_escapes_fail_closed_without_outside_access() {
    let tmp = TempDir::new().unwrap();
    let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_SET_PROJECT_DIR_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();

    for (label, project_dir) in [
        ("traversal", "../outside"),
        ("drive", "C:/outside"),
        ("unc", "//server/share/outside"),
    ] {
        let root = tmp.path().join(format!("{label}-server"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!(
                "include(\":outside\")\nproject(\":outside\").setProjectDir(file(\"{project_dir}\"))\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();

        assert_mcp_gradle_discovery_rejected(
            &root,
            &outside_source,
            outside_bytes,
            "outside",
            label,
        );
    }
}

#[test]
fn mcp_gradle_interpolated_project_dirs_fail_closed_without_outside_access() {
    let tmp = TempDir::new().unwrap();
    let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_GRADLE_INTERPOLATION_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();

    for (label, override_statement) in [
        (
            "assignment-unbraced",
            r#"project(":member").projectDir = file("$outside")"#,
        ),
        (
            "setter-unbraced",
            r#"project(":member").setProjectDir(file("$outside"))"#,
        ),
        (
            "assignment-braced",
            r#"project(":member").projectDir = file("${outside}")"#,
        ),
        (
            "setter-braced",
            r#"project(":member").setProjectDir(file("${outside}"))"#,
        ),
        (
            "indirect-assignment",
            "val member = project(\":member\")\nmember.projectDir = file(\"../outside\")",
        ),
        (
            "indirect-setter",
            "val member = project(\":member\")\nmember.setProjectDir(file(\"../outside\"))",
        ),
        (
            "closure-assignment",
            "project(\":member\") {\nprojectDir = file(\"../outside\")\n}",
        ),
        (
            "whitespace-setter",
            "project(\":member\") . setProjectDir(file(\"../outside\"))",
        ),
        (
            "concatenated-constructor",
            "project(\":member\").projectDir = newFile(rootDir, \"../outside\")",
        ),
        (
            "drive-relative-selector",
            "project(\"C:outside\").projectDir = file(\"modules/member\")",
        ),
        (
            "multiline-conditional-include",
            "if (enabled) {\ninclude(\":member\")\n}",
        ),
        ("triple-quoted-include", "include(\"\"\":member\"\"\")"),
    ] {
        let root = tmp.path().join(format!("{label}-server"));
        setup_minimal_mcp_project_at(&root);
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!("val outside = \"../outside\"\ninclude(\":member\")\n{override_statement}\n"),
        )
        .unwrap();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();

        assert_mcp_gradle_discovery_rejected(
            &root,
            &outside_source,
            outside_bytes,
            "member",
            label,
        );
    }
}

#[cfg(unix)]
#[test]
fn mcp_gradle_symlink_module_escape_fails_closed_without_outside_access() {
    use std::os::unix::fs::symlink;

    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = project_tmp.path();
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_GRADLE_SYMLINK_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    symlink(outside_tmp.path(), root.join("linked")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();

    assert_mcp_gradle_discovery_rejected(root, &outside_source, outside_bytes, "linked", "symlink");
}

fn assert_mcp_gradle_discovery_rejected(
    root: &std::path::Path,
    outside_source: &std::path::Path,
    outside_bytes: &[u8],
    module: &str,
    label: &str,
) {
    let responses = mcp_request_with_write(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_check", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": { "name": "specsync_score", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": { "name": "specsync_generate", "arguments": {} }
            }),
        ],
    );

    assert_eq!(
        responses.len(),
        4,
        "Gradle {label} discovery returned incomplete MCP responses: {responses:#?}"
    );
    for response in &responses {
        assert_eq!(
            response["result"]["isError"], true,
            "Gradle {label} discovery produced an MCP false green: {response}"
        );
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(
            text.contains("Gradle")
                || text.contains("root capability")
                || text.contains("symlink")
                || text.contains("reparse point"),
            "Gradle {label} rejection did not explicitly identify manifest or path confinement: {response}"
        );
        assert!(
            !text.contains("SECRET"),
            "Gradle {label} rejection disclosed outside source bytes: {response}"
        );
    }
    assert!(
        !root.join("specs").join(module).exists(),
        "Gradle {label} rejection allowed MCP generation to create partial output"
    );
    assert_eq!(
        fs::read(outside_source).unwrap(),
        outside_bytes,
        "Gradle {label} rejection changed outside source bytes"
    );
}

#[test]
fn mcp_gradle_preflight_rejects_malformed_settings_without_partial_results() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":member\"\n").unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Gradle")
    );
}

#[cfg(unix)]
#[test]
fn mcp_gradle_preflight_rejects_in_root_manifest_symlinks() {
    use std::os::unix::fs::symlink;

    for manifest in ["build.gradle.kts", "settings.gradle.kts"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let real_manifest = root.join("real-gradle-manifest");
        fs::write(&real_manifest, "plugins {}\n").unwrap();
        symlink(&real_manifest, root.join(manifest)).unwrap();

        let responses = mcp_request(
            &root,
            &[serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            })],
        );

        assert_eq!(responses[0]["result"]["isError"], true);
        assert!(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|error| error.contains("symlink or reparse point")),
            "{manifest}: {responses:#?}"
        );
    }
}

#[test]
fn mcp_gradle_preflight_rejects_every_manifest_above_four_mib() {
    for manifest in [
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
    ] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        fs::write(root.join(manifest), vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();

        let responses = mcp_request(
            &root,
            &[serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            })],
        );

        assert_eq!(responses[0]["result"]["isError"], true);
        assert!(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|error| error.contains("4 MiB")),
            "{manifest}: {responses:#?}"
        );
    }
}

#[test]
fn mcp_gradle_preflight_accepts_comments_and_escaped_paths() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::create_dir_all(root.join("modules/member/src/main/kotlin")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        root.join("settings.gradle.kts"),
        r#"val kotlinDocumentation = """
include(":phantom")
project(":member").projectDir = file("../outside")
"""
def groovyDocumentation = '''
include(":groovy-phantom")
project(":member").setProjectDir(file("../outside"))
'''
/* outer ignored directive:
   include(":outer-phantom")
   /* include(":nested-phantom") */
*/
include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
    )
    .unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );

    assert_ne!(responses[0]["result"]["isError"], true);
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(result["count"], 1);
}
