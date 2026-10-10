use super::*;
use crate::manifest::{MAX_GRADLE_MANIFEST_BYTES, discover_from_manifests_checked};
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use std::fs;
use tempfile::tempdir;

#[test]
fn gradle_manifest_discovery_rejects_non_regular_oversized_and_non_utf8_manifests() {
    for name in [
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
    ] {
        let directory_manifest = tempdir().unwrap();
        fs::create_dir(directory_manifest.path().join(name)).unwrap();
        let error = discover_from_manifests_checked(directory_manifest.path()).unwrap_err();
        assert!(
            error.contains(name) && error.contains("must be a regular file"),
            "unexpected {name} preflight error: {error}"
        );
    }

    let oversized_manifest = tempdir().unwrap();
    fs::write(
        oversized_manifest.path().join("settings.gradle.kts"),
        vec![b'a'; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
    )
    .unwrap();
    let error = discover_from_manifests_checked(oversized_manifest.path()).unwrap_err();
    assert!(error.contains("exceeds the"));

    let non_utf8_manifest = tempdir().unwrap();
    fs::write(
        non_utf8_manifest.path().join("settings.gradle.kts"),
        [0xff, 0xfe],
    )
    .unwrap();
    let error = discover_from_manifests_checked(non_utf8_manifest.path()).unwrap_err();
    assert!(error.contains("not valid UTF-8"));
}

#[test]
fn gradle_manifest_discovery_preflights_shadowed_groovy_variants() {
    for shadowed in ["build.gradle", "settings.gradle"] {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();
        fs::create_dir(tmp.path().join(shadowed)).unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

        assert!(
            error.contains(shadowed) && error.contains("must be a regular file"),
            "unexpected shadowed Gradle manifest error: {error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn gradle_manifest_reads_reject_regular_file_replacement_at_every_checkpoint() {
    for target in [
        GradleManifestReadCheckpoint::PreOpen,
        GradleManifestReadCheckpoint::Opened,
        GradleManifestReadCheckpoint::AfterOpen,
        GradleManifestReadCheckpoint::AfterRead,
    ] {
        let tmp = tempdir().unwrap();
        let manifest = tmp.path().join("settings.gradle.kts");
        let replacement = tmp.path().join("replacement.gradle.kts");
        fs::write(&manifest, "include(\":original\")\n").unwrap();
        fs::write(&replacement, "include(\":replacement\")\n").unwrap();
        let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();
        let mut replaced = false;

        let error = gradle_confined_manifest_text_with_checkpoint(
            &root,
            "settings.gradle.kts",
            "settings",
            |checkpoint| {
                if checkpoint == target && !replaced {
                    fs::rename(&replacement, &manifest).unwrap();
                    replaced = true;
                }
            },
        )
        .unwrap_err();

        assert!(replaced);
        assert!(
            error.contains("changed during confined open")
                || error.contains("changed during confined read"),
            "unexpected identity replacement error at {target:?}: {error}"
        );
    }
}

#[cfg(windows)]
#[test]
fn gradle_manifest_reads_reject_preopen_regular_file_replacement_on_windows() {
    let tmp = tempdir().unwrap();
    let manifest = tmp.path().join("settings.gradle.kts");
    let replacement = tmp.path().join("replacement.gradle.kts");
    fs::write(&manifest, "include(\":original\")\n").unwrap();
    fs::write(&replacement, "include(\":replacement\")\n").unwrap();
    let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();

    let error = gradle_confined_manifest_text_with_checkpoint(
        &root,
        "settings.gradle.kts",
        "settings",
        |checkpoint| {
            if checkpoint == GradleManifestReadCheckpoint::PreOpen {
                fs::remove_file(&manifest).unwrap();
                fs::rename(&replacement, &manifest).unwrap();
            }
        },
    )
    .unwrap_err();

    assert!(error.contains("changed during confined open"));
}
