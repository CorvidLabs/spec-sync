use super::super::super::{validate_spec, validate_spec_content};
use super::validate_companion_scaffold_markers;
use crate::types::{SpecSyncConfig, ValidationResult};
use std::collections::{HashMap, HashSet};
use std::fs;

#[test]
fn companion_scaffold_markers_are_precise_and_ignore_fenced_examples() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let directory = root.join("specs/auth");
    fs::create_dir_all(&directory).unwrap();
    let spec_path = directory.join("auth.spec.md");
    fs::write(&spec_path, "---\nmodule: auth\n---\n").unwrap();
    fs::write(
        directory.join("context.md"),
        "# Context\n\n<!-- Describe the context and motivation for this module. -->\n",
    )
    .unwrap();
    fs::write(
            directory.join("requirements.md"),
            "# Requirements\n\n```markdown\n- <!-- List measurable acceptance criteria. -->\n```\n\nThis prose discusses future acceptance criteria without using the generated marker.\n",
        )
        .unwrap();
    fs::write(
        directory.join("testing.md"),
        "# Testing\n\nList the automated tests and fixtures that protect this module.\n",
    )
    .unwrap();
    fs::write(
            directory.join("design.md"),
            "# Design\n\n## Layout\n\n- Document layout structure, responsive breakpoints, and positioning rules.\n\n## Components\n\n- Document component tree, inputs, outputs, and slots.\n\n## Tokens\n\n- Document color, spacing, typography, and state token overrides.\n\n## Assets\n\n- List icons, images, illustrations, and asset ownership.\n",
        )
        .unwrap();
    let mut result = ValidationResult::new("specs/auth/auth.spec.md".into());

    validate_companion_scaffold_markers(&spec_path, root, &mut result);

    assert_eq!(result.warnings.len(), 6, "{:?}", result.warnings);
    assert!(result.warnings.iter().any(|warning| {
        warning.contains("specs/auth/context.md:3") && warning.contains("concrete motivation")
    }));
    assert!(result.warnings.iter().any(|warning| {
        warning.contains("specs/auth/testing.md:3") && warning.contains("automated tests")
    }));
    for line in [5, 9, 13, 17] {
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains(&format!("specs/auth/design.md:{line}"))),
            "missing design marker warning at line {line}: {:?}",
            result.warnings
        );
    }
    assert!(
        !result
            .warnings
            .iter()
            .any(|warning| warning.contains("requirements.md"))
    );
}

#[test]
fn content_validation_skips_companions_while_path_validation_preserves_them() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let directory = root.join("specs/auth");
    fs::create_dir_all(&directory).unwrap();
    let spec_path = directory.join("auth.spec.md");
    let content = "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n";
    fs::write(&spec_path, content).unwrap();
    fs::write(
        directory.join("context.md"),
        "<!-- Describe the context and motivation for this module. -->\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
    let schema_tables = HashSet::new();
    let schema_columns = HashMap::new();
    let config = SpecSyncConfig::default();

    let path_result = validate_spec(&spec_path, root, &schema_tables, &schema_columns, &config);
    let content_result = validate_spec_content(
        &spec_path,
        content,
        root,
        &schema_tables,
        &schema_columns,
        &config,
    );

    assert!(
        path_result
            .warnings
            .iter()
            .any(|warning| warning.contains("Unfilled companion scaffold marker"))
    );
    assert!(
        content_result
            .warnings
            .iter()
            .all(|warning| !warning.contains("Unfilled companion scaffold marker"))
    );
}
