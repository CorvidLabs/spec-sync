use super::super::super::{compute_coverage, compute_coverage_checked};
#[cfg(unix)]
use super::super::access::coverage_relative_path_text;
use super::super::access::open_coverage_project_root;
#[cfg(unix)]
use super::super::specs::discover_coverage_spec_files;
use super::super::{
    CoverageSourceSnapshot, CoverageTraversalBudget, CoverageTraversalLimits, MAX_COVERAGE_DEPTH,
    MAX_COVERAGE_ENTRIES, MAX_COVERAGE_FILE_BYTES, MAX_COVERAGE_INPUT_BYTES,
};
use super::*;
use crate::types::SpecSyncConfig;
#[cfg(unix)]
use crate::validator::normalize_source_mapping;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::{ffi::OsString, io, path::PathBuf};

#[test]
fn test_compute_coverage_double_star_exclude_no_panic() {
    // Regression: a `**/**` exclude pattern used to panic in the glob matcher
    // (`&pattern[3..len-3]` reverses to `[3..2]` for the len-5 string). It must
    // instead match every path (empty middle) and exclude all source files.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.ts"), "export function a() {}").unwrap();

    let mut config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        exclude_patterns: vec!["**/**".to_string()],
        ..SpecSyncConfig::default()
    };

    // Must not panic; `**/**` excludes all source files.
    let report = compute_coverage(tmp.path(), &[], &config);
    assert_eq!(
        report.total_source_files, 0,
        "`**/**` should exclude every source file"
    );

    // A normal `**/dir/**` pattern still excludes only that directory.
    fs::create_dir_all(src.join("gen")).unwrap();
    fs::write(src.join("gen/z.ts"), "export function z() {}").unwrap();
    config.exclude_patterns = vec!["**/gen/**".to_string()];
    let report = compute_coverage(tmp.path(), &[], &config);
    assert_eq!(
        report.total_source_files, 1,
        "only src/gen should be excluded, leaving src/a.ts"
    );
}

#[cfg(unix)]
#[test]
fn retained_coverage_snapshot_rejects_post_discovery_symlink_replacement() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("member/src/main/kotlin")).unwrap();
    fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
    let outside_source = outside.join("src/main/kotlin/Secret.kt");
    let sentinel = b"const val SECRET = \"RETAINED_COVERAGE_SENTINEL\"\n";
    fs::write(
        root.join("member/src/main/kotlin/Local.kt"),
        "const val LOCAL = 1\n",
    )
    .unwrap();
    fs::write(&outside_source, sentinel).unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["member/src/main/kotlin".to_string()],
        ..SpecSyncConfig::default()
    };

    let project = open_coverage_project_root(&root).unwrap();
    fs::rename(root.join("member"), root.join("original-member")).unwrap();
    symlink(&outside, root.join("member")).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let error = snapshot_coverage_sources(&project, &root, &config, &HashSet::new(), &mut budget)
        .unwrap_err();

    assert!(error.contains("symlink or reparse point"), "{error}");
    assert!(!error.contains("RETAINED_COVERAGE_SENTINEL"));
    assert_eq!(fs::read(outside_source).unwrap(), sentinel);
}

#[test]
fn retained_coverage_sources_reject_regular_directory_replacement_after_selection() {
    let project = tempfile::tempdir().unwrap();
    let original = project.path().join("original-src");
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("src/original.rs"),
        "pub fn original() {}\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };
    let selected_sources = select_coverage_source_directories(
        &retained,
        &config,
        CoverageTraversalLimits::default().max_depth,
    )
    .unwrap();
    fs::rename(project.path().join("src"), &original).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("src/replacement.rs"),
        "pub fn replacement() {}\n",
    )
    .unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let error = snapshot_selected_coverage_sources_with_hook(
        &retained,
        project.path(),
        &config,
        &HashSet::new(),
        &mut budget,
        selected_sources,
        &mut |_| {},
    )
    .unwrap_err();

    assert!(
        error.contains("changed during retained traversal"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn retained_source_traversal_rejects_directory_replacement_after_enumeration() {
    let project = tempfile::tempdir().unwrap();
    let source_parent = project.path().join("src/module");
    let original = source_parent.join("nested");
    let detached = project.path().join("detached-source");
    let replacement = project.path().join("replacement-source");
    fs::create_dir_all(&original).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(original.join("original.rs"), "pub fn original() {}\n").unwrap();
    fs::write(
        replacement.join("replacement.rs"),
        "pub fn replacement() {}\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let mut replaced = false;

    let error = snapshot_coverage_sources_with_hook(
        &retained,
        project.path(),
        &config,
        &HashSet::new(),
        &mut budget,
        |relative| {
            if replaced || relative != Path::new("src/module") {
                return;
            }
            replaced = true;
            fs::rename(&original, &detached).unwrap();
            fs::rename(&replacement, &original).unwrap();
        },
    )
    .unwrap_err();

    assert!(replaced);
    assert!(
        error.contains("changed during retained traversal"),
        "{error}"
    );
}

fn snapshot_with_limits(
    root: &Path,
    config: &SpecSyncConfig,
    exclude_dirs: &HashSet<String>,
    limits: CoverageTraversalLimits,
) -> Result<CoverageSourceSnapshot, String> {
    let project = open_coverage_project_root(root).unwrap();
    let mut budget = CoverageTraversalBudget::new(limits);
    snapshot_coverage_sources(&project, root, config, exclude_dirs, &mut budget)
}

#[cfg(unix)]
#[test]
fn broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit() {
    const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_HANDLES_CHILD";
    if std::env::var_os(CHILD_ENV).is_none() {
        let executable = std::env::current_exe().unwrap();
        let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
        assert!(
            status.success(),
            "broad traversal failed with a 64-descriptor soft limit"
        );
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let source = project.path().join("src");
    let specs = project.path().join("specs");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&specs).unwrap();
    for index in 0..200 {
        let module = source.join(format!("module-{index:03}"));
        fs::create_dir(&module).unwrap();
        fs::write(module.join("lib.rs"), "pub fn visible() {}\n").unwrap();
        let spec_module = specs.join(format!("module-{index:03}"));
        fs::create_dir(&spec_module).unwrap();
        fs::write(
            spec_module.join("module.spec.md"),
            "---\nmodule: broad\nversion: 1\nstatus: stable\nfiles: []\n---\n",
        )
        .unwrap();
    }
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let discovered_specs = discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap();
    assert_eq!(discovered_specs.len(), 200);
    drop(retained);

    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let snapshot = snapshot_with_limits(
        project.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits::default(),
    )
    .unwrap();

    assert_eq!(snapshot.files.len(), 200);
}

#[cfg(unix)]
#[test]
fn broad_configured_source_roots_bound_open_directory_handles() {
    const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_ROOT_HANDLES_CHILD";
    if std::env::var_os(CHILD_ENV).is_none() {
        let executable = std::env::current_exe().unwrap();
        let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_configured_source_roots_bound_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
        assert!(
            status.success(),
            "coverage retained handles across distinct configured roots"
        );
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let mut source_dirs = Vec::new();
    for index in 0..90 {
        let source_dir = format!("source-{index:03}");
        fs::create_dir(project.path().join(&source_dir)).unwrap();
        fs::write(
            project.path().join(&source_dir).join("lib.rs"),
            format!("pub fn visible_{index:03}() {{}}\n"),
        )
        .unwrap();
        source_dirs.push(source_dir);
    }
    let config = SpecSyncConfig {
        source_dirs,
        ..SpecSyncConfig::default()
    };

    let snapshot = snapshot_with_limits(
        project.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits::default(),
    )
    .expect("distinct configured source roots must stay descriptor-bounded");

    assert_eq!(snapshot.files.len(), 90);
}

#[cfg(unix)]
#[test]
fn excluded_special_names_are_skipped_before_metadata() {
    use std::os::unix::fs::symlink;
    use std::os::unix::net::UnixListener;
    use std::process::Command;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
    symlink(&outside, root.join("src/excluded-link")).unwrap();
    let fifo = root.join("src/excluded-fifo");
    let fifo_status = Command::new("mkfifo").arg(&fifo).status().unwrap();
    assert!(fifo_status.success(), "mkfifo fixture failed");
    let _socket = match UnixListener::bind(root.join("src/excluded-socket")) {
        Ok(socket) => Some(socket),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
        Err(error) => panic!("cannot create excluded socket fixture: {error}"),
    };
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        exclude_dirs: vec![
            "excluded-link".to_string(),
            "excluded-fifo".to_string(),
            "excluded-socket".to_string(),
        ],
        ..SpecSyncConfig::default()
    };
    let excludes = config.exclude_dirs.iter().cloned().collect();

    let report = compute_coverage_checked(&root, &[], &config).unwrap();

    assert_eq!(report.total_source_files, 1);
    assert_eq!(report.unspecced_files, ["src/kept.rs"]);
    let snapshot = snapshot_with_limits(
        &root,
        &config,
        &excludes,
        CoverageTraversalLimits::default(),
    )
    .unwrap();
    assert_eq!(snapshot.files.len(), 1);
}

#[cfg(unix)]
#[test]
fn directly_configured_special_source_roots_are_inconclusive() {
    use std::os::unix::fs::symlink;
    use std::os::unix::net::UnixListener;
    use std::process::Command;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("linked")).unwrap();
    let fifo_status = Command::new("mkfifo")
        .arg(root.join("fifo"))
        .status()
        .unwrap();
    assert!(fifo_status.success(), "mkfifo fixture failed");
    let socket = match UnixListener::bind(root.join("socket")) {
        Ok(socket) => Some(socket),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
        Err(error) => panic!("cannot create configured socket fixture: {error}"),
    };
    let mut cases = vec![
        ("linked", "symlink or reparse point"),
        ("fifo", "regular directory"),
    ];
    if socket.is_some() {
        cases.push(("socket", "regular directory"));
    }

    for (configured, expected) in cases {
        let config = SpecSyncConfig {
            source_dirs: vec![configured.to_string()],
            exclude_dirs: vec![configured.to_string()],
            ..SpecSyncConfig::default()
        };
        let error = compute_coverage_checked(&root, &[], &config).unwrap_err();
        assert!(error.contains(expected), "{configured}: {error}");
    }
}

#[cfg(windows)]
fn create_coverage_test_junction(junction: &Path, target: &Path) -> Result<(), String> {
    let unicode_path = |path: &Path| -> Result<String, String> {
        path.to_str()
            .map(str::to_string)
            .ok_or_else(|| "junction fixture paths must be valid Unicode".to_string())
    };
    let junction = unicode_path(junction)?;
    let target = unicode_path(target)?;
    let script = "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction \
                      -Path $env:SPECSYNC_TEST_JUNCTION \
                      -Target $env:SPECSYNC_TEST_TARGET | Out-Null";
    let mut unavailable = Vec::new();
    for executable in ["powershell.exe", "pwsh.exe"] {
        let output = match std::process::Command::new(executable)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
            ])
            .env("SPECSYNC_TEST_JUNCTION", &junction)
            .env("SPECSYNC_TEST_TARGET", &target)
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                unavailable.push(executable);
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to launch {executable} junction fixture: {error}"
                ));
            }
        };
        if output.status.success() {
            return Ok(());
        }
        return Err(format!(
            "{executable} junction fixture exited with {:?}; stdout: {}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Err(format!(
        "failed to launch a PowerShell junction fixture; unavailable executables: {}",
        unavailable.join(", ")
    ))
}

#[cfg(windows)]
#[test]
fn excluded_junction_is_skipped_but_configured_junction_is_inconclusive() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
    let target_proof = b"outside junction target\n";
    fs::write(outside.join("junction-target-proof.txt"), target_proof).unwrap();
    let excluded_junction = root.join("src/excluded-junction");
    let configured_junction = root.join("configured-junction");
    create_coverage_test_junction(&excluded_junction, &outside)
        .unwrap_or_else(|error| panic!("failed to create excluded junction: {error}"));
    create_coverage_test_junction(&configured_junction, &outside)
        .unwrap_or_else(|error| panic!("failed to create configured junction: {error}"));
    for junction in [&excluded_junction, &configured_junction] {
        assert_eq!(
            fs::read(junction.join("junction-target-proof.txt")).unwrap(),
            target_proof,
            "{} does not resolve to the intended outside target",
            junction.display()
        );
    }

    let excluded = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        exclude_dirs: vec!["excluded-junction".to_string()],
        ..SpecSyncConfig::default()
    };
    let report = compute_coverage_checked(&root, &[], &excluded).unwrap();
    assert_eq!(report.total_source_files, 1);

    let configured = SpecSyncConfig {
        source_dirs: vec!["configured-junction".to_string()],
        exclude_dirs: vec!["configured-junction".to_string()],
        ..SpecSyncConfig::default()
    };
    let error = compute_coverage_checked(&root, &[], &configured).unwrap_err();
    assert!(error.contains("symlink or reparse point"), "{error}");
}

#[cfg(unix)]
#[test]
fn unix_backslash_and_directory_separator_paths_do_not_collide() {
    assert_eq!(
        normalize_source_mapping("src/collision\\name.rs"),
        Some("src/collision\\name.rs".to_string())
    );
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src/collision")).unwrap();
    fs::write(
        root.join("src").join("collision\\name.rs"),
        "pub fn literal_backslash() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/collision/name.rs"),
        "pub fn directory_separator() {}\n",
    )
    .unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let report = compute_coverage_checked(root, &[], &config).unwrap();

    assert_eq!(report.total_source_files, 2);
    assert!(
        report
            .unspecced_files
            .contains(&"src/collision\\name.rs".to_string())
    );
    assert!(
        report
            .unspecced_files
            .contains(&"src/collision/name.rs".to_string())
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_source_name_is_inconclusive_instead_of_colliding() {
    use std::os::unix::ffi::OsStringExt;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    let invalid = OsString::from_vec(b"collision-\xff.rs".to_vec());
    let invalid_path = PathBuf::from("src").join(&invalid);
    assert!(coverage_relative_path_text(&invalid_path).is_err());
    if fs::write(root.join("src").join(invalid), "pub fn hidden() {}\n").is_err() {
        return;
    }
    fs::write(
        root.join("src/collision-\u{fffd}.rs"),
        "pub fn visible() {}\n",
    )
    .unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let error = compute_coverage_checked(root, &[], &config).unwrap_err();

    assert!(error.contains("not valid UTF-8"), "{error}");
}

#[test]
fn invalid_utf8_supported_source_content_is_inconclusive() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/bad.rs"), b"pub fn visible() {}\n\xff").unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let error = compute_coverage_checked(root, &[], &config).unwrap_err();

    assert!(error.replace('\\', "/").contains("src/bad.rs"), "{error}");
    assert!(error.contains("not valid UTF-8"), "{error}");
}

#[test]
fn coverage_traversal_enforces_per_file_and_cumulative_byte_limits() {
    assert_eq!(MAX_COVERAGE_FILE_BYTES, 8 * 1024 * 1024);
    assert_eq!(MAX_COVERAGE_INPUT_BYTES, 64 * 1024 * 1024);
    let per_file = tempfile::tempdir().unwrap();
    fs::create_dir_all(per_file.path().join("src")).unwrap();
    fs::write(per_file.path().join("src/large.rs"), b"123456789").unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };
    let per_file_error = snapshot_with_limits(
        per_file.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits {
            max_file_bytes: 8,
            max_input_bytes: 64,
            max_entries: 100,
            max_depth: 16,
        },
    )
    .unwrap_err();
    assert!(per_file_error.contains("8 bytes per-file limit"));

    let cumulative = tempfile::tempdir().unwrap();
    fs::create_dir_all(cumulative.path().join("src")).unwrap();
    fs::write(cumulative.path().join("src/a.rs"), b"123456").unwrap();
    fs::write(cumulative.path().join("src/b.rs"), b"123456").unwrap();
    let cumulative_error = snapshot_with_limits(
        cumulative.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits {
            max_file_bytes: 8,
            max_input_bytes: 10,
            max_entries: 100,
            max_depth: 16,
        },
    )
    .unwrap_err();
    assert!(cumulative_error.contains("10 bytes cumulative limit"));
}

#[test]
fn coverage_traversal_enforces_entry_and_depth_limits_iteratively() {
    assert_eq!(MAX_COVERAGE_ENTRIES, 100_000);
    assert_eq!(MAX_COVERAGE_DEPTH, 256);
    let entries = tempfile::tempdir().unwrap();
    fs::create_dir_all(entries.path().join("src")).unwrap();
    fs::write(entries.path().join("src/a.rs"), "").unwrap();
    fs::write(entries.path().join("src/b.rs"), "").unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };
    let entry_error = snapshot_with_limits(
        entries.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits {
            max_file_bytes: 8,
            max_input_bytes: 16,
            max_entries: 1,
            max_depth: 16,
        },
    )
    .unwrap_err();
    assert!(entry_error.contains("1-entry limit"));

    let depth = tempfile::tempdir().unwrap();
    fs::create_dir_all(depth.path().join("src/a/b/c")).unwrap();
    fs::write(depth.path().join("src/a/b/c/deep.rs"), "").unwrap();
    let depth_error = snapshot_with_limits(
        depth.path(),
        &config,
        &HashSet::new(),
        CoverageTraversalLimits {
            max_file_bytes: 8,
            max_input_bytes: 16,
            max_entries: 100,
            max_depth: 3,
        },
    )
    .unwrap_err();
    assert!(depth_error.contains("3-component depth limit"));
}

#[test]
fn configured_static_html_has_non_vacuous_coverage() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("landing")).unwrap();
    fs::create_dir_all(root.join("specs/landing")).unwrap();
    fs::write(root.join("landing/index.html"), "<main>Welcome</main>\n").unwrap();
    fs::write(root.join("landing/logo.png"), [0_u8, 1, 2]).unwrap();
    let spec_path = root.join("specs/landing/landing.spec.md");
    fs::write(
            &spec_path,
            "---\nmodule: landing\nversion: 1\nstatus: stable\nfiles:\n  - landing/index.html\n---\n\n# Landing\n",
        )
        .unwrap();
    let config = SpecSyncConfig {
        specs_dir: "specs".into(),
        source_dirs: vec!["landing".into()],
        ..SpecSyncConfig::default()
    };

    let mapped = compute_coverage(root, std::slice::from_ref(&spec_path), &config);
    assert_eq!(mapped.total_source_files, 1);
    assert_eq!(mapped.specced_file_count, 1);
    assert_eq!(mapped.file_coverage_percent(), Some(100));
    assert!(mapped.unspecced_files.is_empty());

    let unmapped = compute_coverage(root, &[], &config);
    assert_eq!(unmapped.total_source_files, 1);
    assert_eq!(unmapped.specced_file_count, 0);
    assert_eq!(unmapped.file_coverage_percent(), Some(0));
    assert_eq!(unmapped.unspecced_files, ["landing/index.html"]);
}
