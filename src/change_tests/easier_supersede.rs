// #423: `change supersede` works out the predecessor's acceptance-entry digest itself, says when the
// predecessor is not in this branch's history, and checks the predecessor's entry against the
// successor's base commit when the obligation is declared, naming the commit that moved it.

/// A repository with a legacy `auth` change accepted and committed, its signed `auth` entries, and
/// `moved`: whether `src/auth.rs` is committed again afterwards, outside any change.
fn accepted_auth_predecessor(
    root: &Path,
    moved: bool,
) -> (ChangeRecord, Vec<AcceptanceInputEntryV1>, Option<String>) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut predecessor = completed_section_only_record(root, delta);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the predecessor"]);
    let entries: Vec<AcceptanceInputEntryV1> = load_verification(root, &predecessor)
        .unwrap()
        .acceptance_manifest
        .unwrap()
        .entries
        .into_iter()
        .filter(|entry| entry.path == "src/auth.rs")
        .collect();
    let mover = moved.then(|| {
        fs::write(root.join("src/auth.rs"), "// Moved after acceptance.\n").unwrap();
        quiet_git(root, &["add", "."]);
        quiet_git(root, &["commit", "-m", "move the auth source"]);
        quiet_git(root, &["rev-parse", "HEAD"])
    });
    (predecessor, entries, mover)
}

/// A draft successor that declares `auth` and `src/auth.rs`.
fn auth_successor(root: &Path) -> ChangeRecord {
    let mut successor = current_workflow_record(root, completed_current_record(root));
    successor.affected_paths.push("src/auth.rs".into());
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    successor
}

#[test]
fn supersede_works_out_the_predecessor_entry_digest() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, entries, _) = accepted_auth_predecessor(root, false);
    let successor = auth_successor(root);
    let result = add_supersedes_obligation_with_snapshot(
        root,
        &successor.id,
        &predecessor.id,
        "src/auth.rs",
        "auth",
        None,
    )
    .unwrap();
    let edge = &result.change.supersedes[0];
    assert_eq!(edge.obligations[0].predecessor_entry_digest, entries[0].entry_digest);
    assert_eq!(
        succession_base_warning(root, &successor.id, &predecessor.id, "src/auth.rs").unwrap(),
        None,
        "the base still holds the predecessor's entry"
    );
}

#[test]
fn a_predecessor_missing_from_this_branch_is_named_as_such() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (_, entries, _) = accepted_auth_predecessor(root, false);
    let successor = auth_successor(root);
    let refused = add_supersedes_obligation(
        root,
        &successor.id,
        "feature-a-on-another-branch",
        "src/auth.rs",
        "auth",
        &entries[0].entry_digest,
    )
    .unwrap_err();
    assert!(
        refused.contains("not in this branch's history") && refused.contains("rebase"),
        "{refused}"
    );
}

#[test]
fn a_predecessor_entry_moved_before_the_base_is_named_at_supersede_time() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, _, mover) = accepted_auth_predecessor(root, true);
    let mover = mover.unwrap();
    let successor = auth_successor(root);
    add_supersedes_obligation_with_snapshot(
        root,
        &successor.id,
        &predecessor.id,
        "src/auth.rs",
        "auth",
        None,
    )
    .expect("the obligation is still recorded, for reconcile-history");
    let warning = succession_base_warning(root, &successor.id, &predecessor.id, "src/auth.rs")
        .unwrap()
        .expect("a moved entry is named");
    assert!(
        warning.contains("src/auth.rs")
            && warning.contains(&mover[..12])
            && warning.contains("reconcile-history"),
        "{warning}"
    );
}
