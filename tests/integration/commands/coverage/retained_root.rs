//! Coverage and generation keep their retained roots across path replacement.

use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

#[cfg(windows)]
use super::super::{create_windows_junction, retarget_windows_junction};
use super::{assert_gradle_discovery_is_inconclusive, omit_config_source_dirs};

fn specsync_process() -> std::process::Command {
    let binary = specsync().get_program().to_os_string();
    let mut command = std::process::Command::new(binary);
    command.env_remove("GITHUB_EVENT_NAME");
    command.env_remove("GITHUB_BASE_REF");
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .get(.."SPECSYNC_".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("SPECSYNC_"))
        {
            command.env_remove(key);
        }
    }
    command
}

#[cfg(unix)]
#[test]
fn gradle_symlink_module_escape_is_inconclusive_for_coverage_gating_commands() {
    use std::os::unix::fs::symlink;

    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&project_tmp);
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"GRADLE_SYMLINK_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    symlink(outside_tmp.path(), root.join("linked")).unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();

    assert_gradle_discovery_is_inconclusive(
        &root,
        &outside_source,
        outside_bytes,
        "linked",
        "symlink",
    );
}

// Drives the product's coverage-snapshot rendezvous (`coverage_snapshot_test_barrier`),
// which is `#[cfg(debug_assertions)]`: a release build never publishes the marker, so the
// swap cannot be landed inside the window and the child exits first. Only the rendezvous
// is debug-only — the guard it synchronises (`verify_coverage_project_root`, validator.rs)
// is compiled unconditionally and is present in the shipped binary.
//
// That guard has NO release-runnable test. It is reached only from validator.rs:275 and
// :5440, and no unit test's path covers either call site. Do not mistake
// `validator::tests::retained_coverage_snapshot_rejects_post_discovery_symlink_replacement`
// for substitute coverage: it asserts "symlink or reparse point", which is the
// source-directory symlink refusal, a different guard. Nor
// `retained_coverage_sources_reject_regular_directory_replacement_after_selection`: it does
// assert "changed during retained traversal", but reaches it through
// `snapshot_selected_coverage_sources_with_hook`, so the string comes from the
// source-directory identity checks, never from `verify_coverage_project_root`.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "coverage-snapshot rendezvous is #[cfg(debug_assertions)]"
)]
fn gradle_post_discovery_symlink_swap_is_inconclusive_for_every_coverage_gate() {
    use std::os::unix::fs::symlink;
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
    const BARRIER_PHASE_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "coverage-snapshot-identity";
    for phase in ["root-retained", "manifest-discovered"] {
        for command_name in ["check", "coverage", "generate", "report", "score"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path().join("project");
            let original_root = tmp.path().join("original-project");
            let outside = tmp.path().join("outside");
            let barrier = tmp.path().join("barrier");
            fs::create_dir_all(root.join("member/src/main/kotlin")).unwrap();
            fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
            fs::create_dir_all(&barrier).unwrap();
            fs::write(
                root.join("member/src/main/kotlin/Local.kt"),
                "const val LOCAL = 1\n",
            )
            .unwrap();
            fs::write(root.join("settings.gradle.kts"), "include(\":member\")\n").unwrap();
            let outside_source = outside.join("src/main/kotlin/Secret.kt");
            let outside_bytes = b"const val SECRET = \"POST_DISCOVERY_SWAP\"\n";
            fs::write(&outside_source, outside_bytes).unwrap();

            let mut process = specsync_process();
            process
                .arg(command_name)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"]);
            if command_name == "score" {
                process.args(["--require-coverage", "100"]);
            }
            let mut child = process
                .env(BARRIER_ENV, &barrier)
                .env(BARRIER_PHASE_ENV, phase)
                .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let ready = barrier.join(phase);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready.is_file() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{command_name} exited before the {phase} coverage barrier"
                );
                assert!(
                    Instant::now() < deadline,
                    "{command_name} did not reach the {phase} coverage barrier"
                );
                thread::sleep(Duration::from_millis(10));
            }

            fs::rename(&root, &original_root).unwrap();
            symlink(&outside, &root).unwrap();
            fs::write(barrier.join("resume"), b"resume\n").unwrap();
            let output = child.wait_with_output().unwrap();

            assert!(
                !output.status.success(),
                "{command_name} false green at {phase}"
            );
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!(
                        "{command_name} must emit JSON after a {phase} swap: {error}; stdout={}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                });
            assert_eq!(json["inconclusive"], true, "{command_name}: {json}");
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("project root")
                        && error.contains("changed during retained traversal")),
                "{command_name}: {json}"
            );
            assert!(!String::from_utf8_lossy(&output.stdout).contains("POST_DISCOVERY_SWAP"));
            assert!(!root.join("specs/member").exists());
            assert!(!original_root.join("specs/member").exists());
            assert_eq!(fs::read(&outside_source).unwrap(), outside_bytes);
        }
    }
}

#[cfg(any(unix, windows))]
fn assert_generate_post_coverage_root_retarget_is_inconclusive<CreateAlias, RetargetAlias>(
    create_alias: CreateAlias,
    retarget_alias: RetargetAlias,
) where
    CreateAlias: Fn(&std::path::Path, &std::path::Path) -> Result<(), String>,
    RetargetAlias: Fn(&std::path::Path, &std::path::Path) -> Result<(), String>,
{
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_GENERATE_ROOT_IDENTITY_BARRIER";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "generate-root-identity";
    const MARKER: &str = "coverage-complete";

    const MODULES: [(&str, &[u8], &[u8]); 2] = [
        (
            "member",
            b"pub fn retained_member() {}\n",
            b"pub fn replacement_member() {}\n",
        ),
        (
            "peer",
            b"pub fn retained_peer() {}\n",
            b"pub fn replacement_peer() {}\n",
        ),
    ];

    for (format, batch) in [
        ("text", false),
        ("json", false),
        ("text", true),
        ("json", true),
    ] {
        let case = format!("{format}/{}", if batch { "batch" } else { "default" });
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("requested-project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        let barrier = tmp.path().join("barrier");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::create_dir_all(&barrier).unwrap();
        write_config(&original, "specs", &["src"]);
        write_config(&replacement, "specs", &["src"]);
        let original_config = original.join("specsync.json");
        let original_config_bytes = fs::read(&original_config).unwrap();
        let replacement_config = replacement.join("specsync.json");
        let replacement_config_bytes = fs::read(&replacement_config).unwrap();
        for &(module, original_source_bytes, replacement_source_bytes) in &MODULES {
            let original_source = original.join("src").join(module).join("local.rs");
            let replacement_source = replacement.join("src").join(module).join("replacement.rs");
            fs::create_dir_all(original_source.parent().unwrap()).unwrap();
            fs::create_dir_all(replacement_source.parent().unwrap()).unwrap();
            fs::write(original_source, original_source_bytes).unwrap();
            fs::write(replacement_source, replacement_source_bytes).unwrap();
        }
        let original_sentinel = original.join("original-sentinel.bin");
        let original_sentinel_bytes = b"retained project must remain byte exact\0";
        fs::write(&original_sentinel, original_sentinel_bytes).unwrap();
        let replacement_sentinel = replacement.join("replacement-sentinel.bin");
        let replacement_sentinel_bytes = b"replacement project must remain byte exact\0";
        fs::write(&replacement_sentinel, replacement_sentinel_bytes).unwrap();
        create_alias(&root, &original)
            .unwrap_or_else(|error| panic!("failed to create requested-root alias: {error}"));

        let mut process = specsync_process();
        process
            .arg("generate")
            .arg("--root")
            .arg(&root)
            .args(["--format", format]);
        if batch {
            process.args(["--batch", "member,peer"]);
        }
        let mut child = process
            .env(BARRIER_ENV, &barrier)
            .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let ready = barrier.join(MARKER);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.is_file() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "generate exited before the post-coverage barrier; case={case}"
            );
            assert!(
                Instant::now() < deadline,
                "generate did not reach the post-coverage barrier; case={case}"
            );
            thread::sleep(Duration::from_millis(10));
        }

        retarget_alias(&root, &replacement)
            .unwrap_or_else(|error| panic!("failed to retarget requested-root alias: {error}"));
        fs::write(barrier.join("resume"), b"resume\n").unwrap();
        let output = child.wait_with_output().unwrap();

        assert!(
            !output.status.success(),
            "generate false green after a post-coverage root retarget; case={case}"
        );
        if format == "json" {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!(
                        "generate must emit inconclusive JSON after root retarget: {error}; case={case}; stdout={}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                });
            assert_eq!(json["inconclusive"], true, "case={case}: {json}");
            assert_eq!(
                json["generated"],
                serde_json::json!([]),
                "case={case}: {json}"
            );
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("Generate project root")
                        && error.contains("changed after coverage")),
                "case={case}: {json}"
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("Generation inconclusive:")
                    && stderr.contains("Generate project root")
                    && stderr.contains("changed after coverage"),
                "text generation must report an inconclusive root retarget; case={case}; stderr={stderr}"
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                !stdout.contains("Batch generate complete:") && !stdout.contains("\n  Generated "),
                "text generation falsely reported successful output; case={case}; stdout={stdout}"
            );
        }

        assert_eq!(
            fs::read(&original_config).unwrap(),
            original_config_bytes,
            "generation changed retained config bytes; case={case}"
        );
        assert_eq!(
            fs::read(&replacement_config).unwrap(),
            replacement_config_bytes,
            "generation changed replacement config bytes; case={case}"
        );
        assert_eq!(
            fs::read(&original_sentinel).unwrap(),
            original_sentinel_bytes,
            "generation changed retained sentinel bytes; case={case}"
        );
        assert_eq!(
            fs::read(&replacement_sentinel).unwrap(),
            replacement_sentinel_bytes,
            "generation changed replacement sentinel bytes; case={case}"
        );
        for &(module, original_source_bytes, replacement_source_bytes) in &MODULES {
            assert!(
                !original.join("specs").join(module).exists(),
                "failed generation changed the retained output tree; case={case}; module={module}"
            );
            assert!(
                !replacement.join("specs").join(module).exists(),
                "generation wrote through the replacement root; case={case}; module={module}"
            );
            assert_eq!(
                fs::read(original.join("src").join(module).join("local.rs")).unwrap(),
                original_source_bytes,
                "generation changed retained source bytes; case={case}; module={module}"
            );
            assert_eq!(
                fs::read(replacement.join("src").join(module).join("replacement.rs")).unwrap(),
                replacement_source_bytes,
                "generation changed replacement source bytes; case={case}; module={module}"
            );
        }
    }
}

// Drives the product's post-coverage rendezvous (`generate_after_coverage_test_barrier`),
// which is `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it
// synchronises (`RetainedGenerateRoot::verify_public_path`) is compiled unconditionally
// and is present in the shipped binary.
//
// That guard has no release-runnable test, and no unit coverage in ANY profile:
// src/commands/generate.rs has no `mod tests` at all. This test and its Windows sibling
// are its only coverage, so in release it has none.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "generate post-coverage rendezvous is #[cfg(debug_assertions)]"
)]
fn generate_post_coverage_symlink_root_retarget_is_inconclusive_before_writes() {
    use std::os::unix::fs::symlink;

    assert_generate_post_coverage_root_retarget_is_inconclusive(
        |alias, target| symlink(target, alias).map_err(|error| error.to_string()),
        |alias, target| {
            fs::remove_file(alias).map_err(|error| error.to_string())?;
            symlink(target, alias).map_err(|error| error.to_string())
        },
    );
}

// Same debug-only rendezvous as the symlink variant above, so it carries the same gate.
// This test does run on Windows during RC qualification (release.yml `qualify` →
// `fledge lanes run release-candidate` → `cargo test`, a debug build), where the gate is a
// no-op. It would fail only under `cargo test --release` on Windows, which nothing runs.
#[cfg(windows)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "generate post-coverage rendezvous is #[cfg(debug_assertions)]"
)]
fn generate_post_coverage_junction_root_retarget_is_inconclusive_before_writes() {
    assert_generate_post_coverage_root_retarget_is_inconclusive(
        create_windows_junction,
        retarget_windows_junction,
    );
}

#[cfg(unix)]
#[test]
fn gradle_symlinked_manifests_are_inconclusive_without_reading_outside_bytes() {
    use std::os::unix::fs::symlink;

    for manifest_name in ["build.gradle.kts", "settings.gradle.kts"] {
        let project_tmp = TempDir::new().unwrap();
        let outside_tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&project_tmp);
        let outside_manifest = outside_tmp.path().join(manifest_name);
        let outside_bytes = b"include(\":GRADLE_MANIFEST_SECRET\")\n";
        fs::write(&outside_manifest, outside_bytes).unwrap();
        symlink(&outside_manifest, root.join(manifest_name)).unwrap();
        omit_config_source_dirs(&root);

        for command in ["check", "coverage", "generate", "report", "score"] {
            let output = specsync()
                .arg(command)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"])
                .assert()
                .failure()
                .get_output()
                .stdout
                .clone();
            let json: serde_json::Value =
                serde_json::from_slice(&output).unwrap_or_else(|error| {
                    panic!(
                        "{command} must emit valid JSON for a linked Gradle manifest: {error}; stdout={}",
                        String::from_utf8_lossy(&output)
                    )
                });
            assert_eq!(
                json["inconclusive"], true,
                "unexpected {command} JSON for {manifest_name}: {json}"
            );
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|message| message.contains("symlink or reparse point")),
                "unexpected {command} error for {manifest_name}: {json}"
            );
            assert!(
                !String::from_utf8_lossy(&output).contains("GRADLE_MANIFEST_SECRET"),
                "{command} disclosed outside Gradle manifest bytes"
            );
            assert!(
                !root.join("specs/GRADLE_MANIFEST_SECRET").exists(),
                "{command} mutated output after rejecting {manifest_name}"
            );
            assert_eq!(
                fs::read(&outside_manifest).unwrap(),
                outside_bytes,
                "{command} changed outside Gradle manifest bytes"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn gradle_junction_module_escape_is_inconclusive_for_coverage_gating_commands() {
    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&project_tmp);
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"GRADLE_JUNCTION_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    create_windows_junction(&root.join("linked"), outside_tmp.path())
        .unwrap_or_else(|error| panic!("failed to create Gradle module junction fixture: {error}"));
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();

    assert_gradle_discovery_is_inconclusive(
        &root,
        &outside_source,
        outside_bytes,
        "linked",
        "junction",
    );
}

// Same debug-only coverage-snapshot rendezvous as the symlink variant above, so it carries
// the same gate. Like the junction variant above, it does run on Windows during RC
// qualification (a debug build), where the gate is a no-op.
#[cfg(windows)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "coverage-snapshot rendezvous is #[cfg(debug_assertions)]"
)]
fn gradle_post_discovery_junction_swap_is_inconclusive_for_every_coverage_gate() {
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
    const BARRIER_PHASE_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "coverage-snapshot-identity";
    for phase in ["root-retained", "manifest-discovered"] {
        for command_name in ["check", "coverage", "generate", "report", "score"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path().join("project");
            let original_root = tmp.path().join("original-project");
            let outside = tmp.path().join("outside");
            let barrier = tmp.path().join("barrier");
            fs::create_dir_all(original_root.join("member/src/main/kotlin")).unwrap();
            fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
            fs::create_dir_all(&barrier).unwrap();
            fs::write(
                original_root.join("member/src/main/kotlin/Local.kt"),
                "const val LOCAL = 1\n",
            )
            .unwrap();
            fs::write(
                original_root.join("settings.gradle.kts"),
                "include(\":member\")\n",
            )
            .unwrap();
            let outside_source = outside.join("src/main/kotlin/Secret.kt");
            let outside_bytes = b"const val SECRET = \"POST_DISCOVERY_JUNCTION_SWAP\"\n";
            fs::write(&outside_source, outside_bytes).unwrap();
            create_windows_junction(&root, &original_root).unwrap_or_else(|error| {
                panic!("failed to create initial project-root junction fixture: {error}")
            });
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(&original_root).unwrap(),
                "initial project-root junction does not target the local project"
            );

            let mut process = specsync_process();
            process
                .arg(command_name)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"]);
            if command_name == "score" {
                process.args(["--require-coverage", "100"]);
            }
            let mut child = process
                .env(BARRIER_ENV, &barrier)
                .env(BARRIER_PHASE_ENV, phase)
                .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let ready = barrier.join(phase);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready.is_file() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{command_name} exited before the {phase} coverage barrier"
                );
                assert!(
                    Instant::now() < deadline,
                    "{command_name} did not reach the {phase} coverage barrier"
                );
                thread::sleep(Duration::from_millis(10));
            }

            retarget_windows_junction(&root, &outside).unwrap_or_else(|error| {
                panic!("failed to retarget post-discovery project-root junction fixture: {error}")
            });
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(&outside).unwrap(),
                "replacement project-root junction does not target the outside directory"
            );
            fs::write(barrier.join("resume"), b"resume\n").unwrap();
            let output = child.wait_with_output().unwrap();

            assert!(
                !output.status.success(),
                "{command_name} false green at {phase}"
            );
            let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!(
                    "{command_name} must emit JSON after a {phase} junction swap: {error}; stdout={}",
                    String::from_utf8_lossy(&output.stdout)
                )
            });
            assert_eq!(json["inconclusive"], true, "{command_name}: {json}");
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("project root")
                        && error.contains("changed during retained traversal")),
                "{command_name}: {json}"
            );
            assert!(
                !String::from_utf8_lossy(&output.stdout).contains("POST_DISCOVERY_JUNCTION_SWAP")
            );
            assert!(!root.join("specs/member").exists());
            assert!(!original_root.join("specs/member").exists());
            assert_eq!(fs::read(&outside_source).unwrap(), outside_bytes);
        }
    }
}
