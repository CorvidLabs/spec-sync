use super::super::parse_gradle;
use crate::manifest::{discover_from_manifests, discover_from_manifests_checked};
use std::fs;
use tempfile::tempdir;

#[test]
fn gradle_settings_only_workspace_discovers_included_modules() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("member/src/main/kotlin")).unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        "include(\":member\")\n",
    )
    .unwrap();

    let result = discover_from_manifests_checked(tmp.path()).unwrap();

    assert_eq!(
        result.modules["member"].source_paths,
        vec!["member/src/main/kotlin"]
    );
}

#[test]
fn gradle_settings_only_workspace_fails_closed_when_malformed() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("settings.gradle"), "include(\":member\"\n").unwrap();

    let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

    assert!(error.contains("Cannot parse Gradle settings manifest"));
    assert!(discover_from_manifests(tmp.path()).modules.is_empty());
}

#[test]
fn gradle_manifest_discovery_rejects_project_root_escape_without_partial_modules() {
    let tmp = tempdir().unwrap();
    let project = tmp.path().join("project");
    let outside = tmp.path().join("outside/src/main/kotlin");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        project.join("settings.gradle.kts"),
        "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n",
    )
    .unwrap();

    let error = discover_from_manifests_checked(&project).unwrap_err();
    assert!(error.contains("must remain beneath the project root"));
    let compatibility_result = discover_from_manifests(&project);
    assert!(compatibility_result.modules.is_empty());
    assert!(compatibility_result.source_dirs.is_empty());
}

#[cfg(unix)]
#[test]
fn gradle_manifest_discovery_rejects_symlinked_module_directories() {
    use std::os::unix::fs::symlink;

    let tmp = tempdir().unwrap();
    let project = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
    symlink(&outside, project.join("linked")).unwrap();
    fs::write(
        project.join("settings.gradle.kts"),
        "include(\":linked\")\n",
    )
    .unwrap();

    let error = discover_from_manifests_checked(&project).unwrap_err();
    assert!(
        error.contains("must not traverse a symlink or reparse point"),
        "unexpected Gradle symlink rejection: {error}"
    );
}

#[test]
fn gradle_single_project_uses_directory_name_not_package_tld() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("payments");
    fs::create_dir_all(root.join("src/main/kotlin/com/example/tool")).unwrap();
    fs::write(
        root.join("build.gradle.kts"),
        "plugins { kotlin(\"jvm\") }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/tool/Profile.kt"),
        "package com.example.tool\nclass Profile\n",
    )
    .unwrap();

    let result = parse_gradle(&root).unwrap();
    assert!(
        result.modules.contains_key("payments"),
        "single-project Gradle with no rootProject.name must use the directory name: {:?}",
        result.modules.keys().collect::<Vec<_>>()
    );
    assert!(
        !result.modules.contains_key("com"),
        "package TLD must not become a module: {:?}",
        result.modules.keys().collect::<Vec<_>>()
    );
    assert!(
        !result.modules.contains_key("src"),
        "shared src/ leaf must not become a module: {:?}",
        result.modules.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        result.modules["payments"].source_paths,
        vec!["src/main/kotlin"]
    );
}

#[test]
fn gradle_single_project_uses_literal_root_project_name() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("checkout-dir");
    fs::create_dir_all(root.join("src/main/kotlin/com/example")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        root.join("settings.gradle.kts"),
        "rootProject.name = \"checkout\"\n",
    )
    .unwrap();
    fs::write(root.join("src/main/kotlin/com/example/A.kt"), "class A\n").unwrap();

    let result = parse_gradle(&root).unwrap();
    assert!(result.modules.contains_key("checkout"));
    assert!(!result.modules.contains_key("checkout-dir"));
    assert!(!result.modules.contains_key("com"));
    assert_eq!(result.modules.len(), 1);
}

#[test]
fn gradle_multi_project_keeps_include_names_and_does_not_add_root() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("workspace");
    fs::create_dir_all(root.join("packages/profile/src/main/kotlin/com/example")).unwrap();
    fs::create_dir_all(root.join("packages/provider/src/main/kotlin/com/example")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
            root.join("settings.gradle.kts"),
            "rootProject.name = \"workspace\"\ninclude(\":profile\", \":provider\")\nproject(\":profile\").projectDir = file(\"packages/profile\")\nproject(\":provider\").projectDir = file(\"packages/provider\")\n",
        )
        .unwrap();
    fs::write(
        root.join("packages/profile/src/main/kotlin/com/example/Profile.kt"),
        "class Profile\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/provider/src/main/kotlin/com/example/Provider.kt"),
        "class Provider\n",
    )
    .unwrap();

    let result = parse_gradle(&root).unwrap();
    assert!(result.modules.contains_key("profile"));
    assert!(result.modules.contains_key("provider"));
    assert!(!result.modules.contains_key("workspace"));
    assert!(!result.modules.contains_key("com"));
    assert!(!result.modules.contains_key("src"));
    assert_eq!(result.modules.len(), 2);
}

#[test]
fn gradle_manifest_discovery_uses_the_effective_project_directory() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("vendor/custom/src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle"),
        "include ':member'\nproject(':member').projectDir = file('vendor/custom')\n",
    )
    .unwrap();

    let result = parse_gradle(tmp.path()).unwrap();
    assert_eq!(
        result.modules["member"].source_paths,
        vec!["vendor/custom/src/main/kotlin"]
    );
    assert!(
        result
            .source_dirs
            .contains(&"vendor/custom/src/main/kotlin".to_string())
    );
}

#[test]
fn gradle_manifest_discovery_fails_closed_for_malformed_settings() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        "include(\":member\"\n",
    )
    .unwrap();

    let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
    assert!(error.contains("Cannot parse Gradle settings manifest"));
    let result = discover_from_manifests(tmp.path());
    assert!(result.modules.is_empty());
    assert!(result.source_dirs.is_empty());
}

#[test]
fn gradle_manifest_discovery_accepts_comments_and_escaped_paths() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("modules/member/src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        r#"include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
    )
    .unwrap();

    let result = discover_from_manifests(tmp.path());
    assert_eq!(
        result.modules["member"].source_paths,
        vec!["modules/member/src/main/kotlin"]
    );
}
