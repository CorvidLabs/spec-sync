use super::super::access::open_coverage_project_root;
use super::super::{CoverageSpecFile, CoverageTraversalBudget, CoverageTraversalLimits};
use super::*;
use crate::types::SpecSyncConfig;
use crate::validator::compute_coverage_checked;
use std::fs;
use std::path::Path;

#[cfg(unix)]
#[test]
fn retained_spec_traversal_rejects_directory_replacement_after_enumeration() {
    let project = tempfile::tempdir().unwrap();
    let spec_parent = project.path().join("specs/module");
    let original = spec_parent.join("nested");
    let detached = project.path().join("detached-specs");
    let replacement = project.path().join("replacement-specs");
    fs::create_dir_all(&original).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(
        original.join("original.spec.md"),
        "---\nmodule: original\nversion: 1\nstatus: stable\nfiles: []\n---\n",
    )
    .unwrap();
    fs::write(
        replacement.join("replacement.spec.md"),
        "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles: []\n---\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let mut replaced = false;

    let error =
        discover_coverage_spec_files_with_hook(&retained, "specs", &mut budget, |relative| {
            if replaced || relative != Path::new("specs/module") {
                return;
            }
            replaced = true;
            fs::rename(&original, &detached).unwrap();
            fs::rename(&replacement, &original).unwrap();
        })
        .unwrap_err();

    assert!(replaced);
    assert!(
        error.contains("changed during retained traversal"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn retained_coverage_spec_mapping_ignores_an_ambient_root_replacement() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let original = tmp.path().join("original-project");
    let replacement = tmp.path().join("replacement-project");
    let relative_spec = Path::new("specs/auth/auth.spec.md");
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(replacement.join("specs/auth")).unwrap();
    fs::write(
        root.join(relative_spec),
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/retained.rs\n---\n",
    )
    .unwrap();
    fs::write(
        replacement.join(relative_spec),
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
    )
    .unwrap();
    let project = open_coverage_project_root(&root).unwrap();
    let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let inventory = discover_coverage_spec_files(&project, "specs", &mut inventory_budget).unwrap();
    fs::rename(&root, &original).unwrap();
    symlink(&replacement, &root).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let mappings = collect_specced_files(&project, &inventory, &mut budget).unwrap();

    assert!(mappings.contains("src/retained.rs"));
    assert!(!mappings.contains("src/replacement.rs"));
}

#[test]
fn retained_coverage_spec_mapping_rejects_a_path_outside_the_project() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();

    let error = select_coverage_spec_files(
        project.path(),
        &[outside.path().join("outside.spec.md")],
        &[],
    )
    .unwrap_err();

    assert!(
        error.contains("must remain beneath the project root"),
        "{error}"
    );
}

#[test]
fn retained_selected_specs_enforce_the_shared_entry_budget() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("specs")).unwrap();
    let first = project.path().join("specs/first.spec.md");
    let second = project.path().join("specs/second.spec.md");
    let spec = "---\nmodule: fixture\nversion: 1\nstatus: stable\nfiles: []\n---\n";
    fs::write(&first, spec).unwrap();
    fs::write(&second, spec).unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let inventory =
        discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
    let limits = CoverageTraversalLimits {
        max_entries: 1,
        ..CoverageTraversalLimits::default()
    };
    let mut within_budget = CoverageTraversalBudget::new(limits);
    collect_specced_files(
        &retained,
        std::slice::from_ref(&inventory[0]),
        &mut within_budget,
    )
    .unwrap();

    let mut over_budget = CoverageTraversalBudget::new(limits);
    let error = collect_specced_files(&retained, &inventory, &mut over_budget).unwrap_err();

    assert!(error.contains("1-entry limit"), "{error}");
}

#[test]
fn retained_spec_enumeration_is_bounded_before_returning_paths() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("specs")).unwrap();
    fs::write(project.path().join("specs/first.spec.md"), "").unwrap();
    fs::write(project.path().join("specs/second.spec.md"), "").unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits {
        max_entries: 1,
        ..CoverageTraversalLimits::default()
    });

    let error = discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap_err();

    assert!(error.contains("1-entry limit"), "{error}");
}

#[test]
fn retained_spec_inventory_applies_caller_selection_before_ownership_parsing() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("specs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    let first = project.path().join("specs/first.spec.md");
    let second = project.path().join("specs/second.spec.md");
    fs::write(
        &first,
        "---\nmodule: first\nversion: 1\nstatus: stable\nfiles:\n  - src/first.rs\n---\n",
    )
    .unwrap();
    fs::write(
        &second,
        "---\nmodule: second\nversion: 1\nstatus: stable\nfiles:\n  - src/second.rs\n---\n",
    )
    .unwrap();
    fs::write(project.path().join("src/first.rs"), "pub fn first() {}\n").unwrap();
    fs::write(project.path().join("src/second.rs"), "pub fn second() {}\n").unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let report =
        compute_coverage_checked(project.path(), std::slice::from_ref(&first), &config).unwrap();

    assert_eq!(report.total_source_files, 2);
    assert_eq!(report.specced_file_count, 1);
    assert_eq!(report.unspecced_files, ["src/second.rs"]);
}

#[test]
fn retained_spec_inventory_rejects_replacement_before_ownership_parse() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("specs")).unwrap();
    let selected = project.path().join("specs/selected.spec.md");
    let replacement = project.path().join("specs/replacement.spec.md");
    fs::write(
        &selected,
        "---\nmodule: selected\nversion: 1\nstatus: stable\nfiles:\n  - src/original.rs\n---\n",
    )
    .unwrap();
    fs::write(
            &replacement,
            "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
        )
        .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let inventory =
        discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
    let selected_inventory: Vec<CoverageSpecFile> = inventory
        .into_iter()
        .filter(|spec| spec.relative_path == Path::new("specs/selected.spec.md"))
        .collect();
    let mut read_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let mut replaced = false;

    let error =
        collect_specced_files_with_hook(&retained, &selected_inventory, &mut read_budget, |_| {
            if replaced {
                return;
            }
            replaced = true;
            fs::remove_file(&selected).unwrap();
            fs::rename(&replacement, &selected).unwrap();
        })
        .unwrap_err();

    assert!(
        error.contains("changed after retained inventory"),
        "{error}"
    );
}
