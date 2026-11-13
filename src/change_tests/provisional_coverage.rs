// #657: an accepted change's changed input counts as covered for now when a later change that is
// approved, implementing or verifying declares its owner module and path. Audit then warns,
// naming that change, instead of failing; the accepted change may archive, and its archive
// records which change covered the file. With no such change the input is stale, as before.

/// A legacy `auth` change accepted and committed, then `src/auth.rs` edited and committed. With
/// `successor`, an approved change created after it declares `auth` and `src/auth.rs`.
fn accepted_change_with_moved_input(root: &Path, successor: bool) -> (ChangeRecord, Option<ChangeRecord>) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut accepted = completed_section_only_record(root, delta);
    accepted = approve_definition(root, &accepted.id, Some("Reviewer".into()), None).unwrap();
    accepted = start_implementation(root, &accepted.id).unwrap();
    verify_change(root, &accepted.id).unwrap();
    accepted = accept_change(root, &accepted.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the first change"]);
    let in_flight = successor.then(|| {
        let mut later = completed_current_record(root);
        for artifact in &later.selected_artifacts {
            let content = if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Edit the shared source.\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
            };
            fs::write(change_dir(root, &later.id).join(artifact.file_name()), content).unwrap();
        }
        fs::write(
            delta_path(root, &later, "auth"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the later change.\n",
        )
        .unwrap();
        later.affected_paths.push("src/auth.rs".into());
        later.affected_paths.sort();
        later.affected_paths.dedup();
        save_change(root, &later).unwrap();
        write_change_markdown(root, &later).unwrap();
        approve_definition(root, &later.id, Some("Scope owner".into()), None).unwrap()
    });
    fs::write(root.join("src/auth.rs"), "// Edited by the later change.\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "the later change edits the shared source"]);
    (accepted, in_flight)
}

fn terminal_validity(root: &Path, record: &ChangeRecord) -> TerminalEvidenceSummary {
    terminal_evidence_summary_with_records(root, record, &list_all_changes_checked(root).unwrap())
}

#[test]
fn an_in_flight_change_that_declares_the_input_covers_it_for_now() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();

    let evidence = terminal_validity(root, &accepted);
    assert_eq!(evidence.validity.as_str(), "provisionally-covered", "{evidence:?}");
    assert!(
        evidence.reason.as_deref().is_some_and(|reason| reason.contains(&in_flight.id)),
        "{evidence:?}"
    );
    let report = audit_project(root);
    assert!(
        !report.errors.iter().any(|error| error.contains(&accepted.id)),
        "{:?}",
        report.errors
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains(&accepted.id) && warning.contains(&in_flight.id)),
        "{:?}",
        report.warnings
    );

    let archived = archive_change(root, &accepted.id).expect("a provisionally covered change may archive");
    let coverage: serde_json::Value =
        serde_json::from_slice(&fs::read(archived.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(coverage["covered_by"][0]["path"], "src/auth.rs", "{coverage}");
    assert_eq!(coverage["covered_by"][0]["change"], in_flight.id.as_str(), "{coverage}");
}

#[test]
fn with_no_change_declaring_the_input_it_is_stale_as_before() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, false);
    assert_eq!(terminal_validity(root, &accepted).validity.as_str(), "stale");
    let report = audit_project(root);
    assert!(
        report.errors.iter().any(|error| error.contains(&accepted.id) && error.contains("stale")),
        "{:?}",
        report.errors
    );
}
