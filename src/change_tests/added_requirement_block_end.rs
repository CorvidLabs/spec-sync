// Issue #1071: validation and the applier read a requirement's block the same way, ending it at
// the next heading of level 3 or shallower. A `##` section after an unchanged ADDED requirement
// is not part of that requirement, so validation no longer calls the requirement a conflict
// while the applier treats it as applied.

mod added_requirement_block_end {
    use super::*;

const SPEC: &str = "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n| `auth` | Auth entry. |\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n";

/// The living requirements: REQ-auth-001 as an ADDED delta materializes it, then a `## Notes`
/// section a person added below it.
const REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe auth module SHALL provide an `auth` entry point.\n\nAcceptance Criteria\n- Callers can invoke `auth`.\n\n## Notes\n\nA later module note, not part of REQ-auth-001.\n";

fn record_with_delta(root: &Path, delta: &str) -> ChangeRecord {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), SPEC).unwrap();
    fs::write(root.join("specs/auth/requirements.md"), REQUIREMENTS).unwrap();
    let record = current_workflow_record(root, completed_record(root));
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Complete\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
            },
        )
        .unwrap();
    }
    fs::write(delta_path(root, &record, "auth"), delta).unwrap();
    record
}

/// An unchanged ADDED requirement followed by a `##` section validates, and the applier reads
/// the same block as already applied.
#[test]
fn an_unchanged_added_requirement_followed_by_a_section_is_not_a_conflict() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = record_with_delta(
        root,
        "## ADDED\n### REQUIREMENT REQ-auth-001\nThe auth module SHALL provide an `auth` entry point.\n\nAcceptance Criteria\n- Callers can invoke `auth`.\n",
    );
    validate_delta_files(root, &record)
        .unwrap_or_else(|error| panic!("an unchanged ADDED requirement is a conflict: {error}"));
    let reapplied = apply_markdown_block(
        REQUIREMENTS,
        "### ",
        "REQ-auth-001",
        "The auth module SHALL provide an `auth` entry point.\n\nAcceptance Criteria\n- Callers can invoke `auth`.",
        DeltaOperation::Added,
    )
    .unwrap();
    assert_eq!(reapplied, REQUIREMENTS, "the applier sees it applied");
}

/// A different body under the same id is still a conflict, with the section after it.
#[test]
fn a_changed_added_requirement_followed_by_a_section_is_still_a_conflict() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = record_with_delta(
        root,
        "## ADDED\n### REQUIREMENT REQ-auth-001\nThe auth module SHALL provide a different entry point.\n\nAcceptance Criteria\n- Callers can invoke `auth`.\n",
    );
    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(
        error.contains("cannot add existing block `REQ-auth-001`"),
        "{error}"
    );
}
}
