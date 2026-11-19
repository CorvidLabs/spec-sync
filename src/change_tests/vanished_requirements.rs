// #1081 (draft, red first): a change never deletes a canonical requirement that no approved
// `## REMOVED` delta names. In CorvidLabs/starling PR #43 a setup script truncated
// `requirements.md` to the change's one added block, and every check passed.

const VANISHED_BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\n### REQ-auth-002\n\nThe system SHALL expire a session after one day.\n";

/// The starling shape: the change adds REQ-auth-003, is approved and pushed as the default
/// branch's state, and then the branch's `requirements.md` is truncated to the header and the
/// added block. `change check` must refuse before writing, naming REQ-auth-001 and REQ-auth-002.
#[test]
fn a_branch_that_lost_requirements_no_removed_delta_names_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(
        root,
        "## ADDED\n\n### REQUIREMENT REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n",
    );
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "approved"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    // The setup script truncates the file before appending the new block.
    let truncated = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n";
    fs::write(requirements_path(root), truncated).unwrap();
    git(&["commit", "-q", "-am", "truncate and append"]);

    let error = materialize_change_deltas(root, &record.id)
        .expect_err("a branch that lost REQ-auth-001 and REQ-auth-002 must be refused");

    assert!(
        error.contains("REQ-auth-001") && error.contains("REQ-auth-002"),
        "{error}"
    );
    assert!(error.contains("REMOVED"), "{error}");
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), truncated);
}
