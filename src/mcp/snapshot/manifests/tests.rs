use super::super::ProjectSnapshot;
use super::*;
use crate::mcp::tests::{open_test_directory, setup_project};
use crate::mcp::{MAX_CONFINEMENT_ENTRIES, MAX_PROJECT_FILE_BYTES};
use std::fs;
use tempfile::TempDir;

#[test]
fn snapshot_includes_manifest_derived_member_under_ignored_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
}

#[test]
fn snapshot_normalizes_confined_cargo_sibling_dependency() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/a\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/b/Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
}

#[test]
fn snapshot_normalizes_confined_windows_native_cargo_paths() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = ['crates\\a']\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/b/Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("crates/a/src/lib.rs").is_file());
    assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
}

#[test]
fn snapshot_ignores_nonsemantic_cargo_metadata_paths() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor")).unwrap();
    fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"metadata-path\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"vendor/lib.rs\"\n\n[package.metadata.example]\npath = \"/benign/absolute/metadata/value\"\n",
        )
        .unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(
        tmp.path().join("vendor/lib.rs"),
        "pub fn custom_target() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path())
        .expect("non-semantic Cargo metadata paths must not affect snapshot discovery");

    assert!(snapshot.root().join("src/lib.rs").is_file());
    assert!(snapshot.root().join("vendor/lib.rs").is_file());
}

#[test]
fn snapshot_manifest_input_rejects_true_root_and_cross_platform_lexical_escapes() {
    let error = snapshot_manifest_input(
        Path::new("crates/a"),
        "../../../outside",
        "Cargo target path",
    )
    .expect_err("parent traversal beyond the retained root must be rejected");
    assert!(error.contains("escapes the configured server root"));

    for configured in [
        "/outside",
        "C:/outside",
        "C:outside",
        r"\\server\share\outside",
        r"\outside",
    ] {
        let error = snapshot_manifest_input(Path::new("crates/a"), configured, "Cargo target path")
            .expect_err("absolute, drive, UNC, and backslash paths must be rejected portably");
        assert!(error.contains("safe project-relative path"));
    }

    let error = snapshot_manifest_input(
        Path::new("crates/a"),
        r"..\..\..\outside",
        "Cargo target path",
    )
    .expect_err("Windows-native traversal beyond the retained root must be rejected");
    assert!(error.contains("escapes the configured server root"));
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_confined_sibling_dependency_symlink_escape() {
    use std::os::unix::fs::symlink;

    let tmp = setup_project();
    let outside = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(outside.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/a\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
    )
    .unwrap();
    fs::write(
        outside.path().join("Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(outside.path().join("src/lib.rs"), "pub fn outside() {}\n").unwrap();
    symlink(outside.path(), tmp.path().join("crates/b")).unwrap();

    ProjectSnapshot::create(tmp.path())
        .err()
        .expect("a normalized sibling dependency must not follow a symlink outside root");
}

#[test]
fn snapshot_includes_multiline_cargo_members_beneath_ignored_directories() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"vendor/member\", # kept\n]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::File::create(tmp.path().join("vendor/member/src/lib.rs"))
        .unwrap()
        .set_len(MAX_PROJECT_FILE_BYTES + 1)
        .unwrap();

    let error = ProjectSnapshot::create(tmp.path())
        .err()
        .expect("a multiline workspace member must remain visible and bounded");

    assert!(error.contains("per-file limit"));
    let expected_path = PathBuf::from("vendor")
        .join("member")
        .join("src")
        .join("lib.rs")
        .display()
        .to_string();
    assert!(error.contains(&expected_path));
}

#[test]
fn snapshot_uses_toml_structure_when_comments_precede_workspace_header() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "# [workspace]\n# members = [\"ignored\"]\n\n[workspace]\nmembers = [\"vendor/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
}

#[test]
fn snapshot_rejects_malformed_cargo_toml_without_partial_discovery() {
    let tmp = setup_project();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/member\"\n",
    )
    .unwrap();

    let error = ProjectSnapshot::create(tmp.path())
        .err()
        .expect("malformed Cargo TOML must make snapshot discovery inconclusive");

    assert!(error.contains("Cannot parse MCP Cargo workspace manifest Cargo.toml as TOML"));
}

#[test]
fn manifest_discovery_charges_the_shared_cumulative_snapshot_budget() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("vendor/one")).unwrap();
    fs::create_dir_all(tmp.path().join("vendor/two")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/one\", \"vendor/two\"]\n",
    )
    .unwrap();
    fs::write(tmp.path().join("vendor/one/Cargo.toml"), vec![b' '; 300]).unwrap();
    fs::write(tmp.path().join("vendor/two/Cargo.toml"), vec![b' '; 300]).unwrap();
    let source = open_test_directory(tmp.path());
    let mut inputs = Vec::new();
    let mut budget = SnapshotBudget {
        max_bytes: 512,
        ..SnapshotBudget::default()
    };

    let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
        .expect_err("manifest parsing must not bypass the cumulative byte budget");

    assert!(error.contains("512 bytes cumulative limit"));
}

#[test]
fn snapshot_workspace_declarations_are_charged_before_deduplication() {
    let cargo = TempDir::new().unwrap();
    fs::create_dir_all(cargo.path().join("crates/member")).unwrap();
    fs::write(
        cargo.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        cargo.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let cargo_source = open_test_directory(cargo.path());
    let mut cargo_inputs = Vec::new();
    let mut cargo_budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };

    collect_snapshot_manifest_inputs(&cargo_source, &mut cargo_inputs, &mut cargo_budget)
        .expect("two duplicate Cargo declarations must fit exactly at the traversal limit");

    assert_eq!(cargo_budget.entries, MAX_CONFINEMENT_ENTRIES);
    assert_eq!(
        cargo_inputs
            .iter()
            .filter(|path| path.as_path() == Path::new("crates/member"))
            .count(),
        1
    );

    let node = TempDir::new().unwrap();
    fs::write(
        node.path().join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    let node_source = open_test_directory(node.path());
    let mut node_inputs = Vec::new();
    let mut node_budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };

    let error = collect_snapshot_manifest_inputs(&node_source, &mut node_inputs, &mut node_budget)
        .expect_err("a duplicate Node declaration beyond the traversal limit must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
    assert_eq!(node_budget.entries, MAX_CONFINEMENT_ENTRIES + 1);
    assert_eq!(
        node_inputs
            .iter()
            .filter(|path| path.as_path() == Path::new("packages"))
            .count(),
        0
    );
}

#[test]
fn snapshot_package_json_fails_closed_and_charges_entries_before_type_validation() {
    for (content, expected) in [
        ("{", "Cannot parse MCP package.json as JSON"),
        ("[]", "root must be a JSON object"),
        (
            r#"{"workspaces":7}"#,
            "`workspaces` must be an array or an object",
        ),
        (
            r#"{"workspaces":{"packages":7}}"#,
            "`workspaces.packages` must be an array",
        ),
        (
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), content).unwrap();
        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("invalid package.json must make snapshot discovery inconclusive");
        assert!(error.contains(expected), "{content}: {error}");
    }

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":[7,"packages/*"]}"#,
    )
    .unwrap();
    let source = open_test_directory(tmp.path());
    let mut inputs = Vec::new();
    let mut budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };
    let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
        .expect_err("wrong-typed workspace entries must fail after declaration charging");

    assert!(
        error.contains("workspace entries must be strings"),
        "{error}"
    );
    assert_eq!(budget.entries, MAX_CONFINEMENT_ENTRIES);
}

#[test]
fn snapshot_includes_python_package_derived_under_ignored_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("vendor")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("pyproject.toml"),
        "[project]\nname = \"vendor\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("vendor/module.py"), "def public(): pass\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    assert!(snapshot.root().join("vendor/module.py").is_file());
}

#[test]
fn snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle"),
        r#"
include ':vendor:groovy',
        ':nested:second'
include(
    ":kotlin:member",
    ':override'
)
project(':override').projectDir = file('vendor/custom')
"#,
    )
    .unwrap();
    for path in [
        "vendor/groovy/src/main/kotlin/one.kt",
        "nested/second/src/main/java/Two.java",
        "kotlin/member/src/main/kotlin/three.kt",
        "vendor/custom/src/main/kotlin/four.kt",
    ] {
        let path = tmp.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "public class Module {}\n").unwrap();
    }

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(
        snapshot
            .root()
            .join("vendor/groovy/src/main/kotlin/one.kt")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("nested/second/src/main/java/Two.java")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("kotlin/member/src/main/kotlin/three.kt")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("vendor/custom/src/main/kotlin/four.kt")
            .is_file()
    );
}
