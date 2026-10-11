
// Honest label: DISCRIMINATOR for #716, the other end of the same mistake and the destructive
// one. A trailing space on the CLOSING delimiter sent the scan past the real end of the block and
// stopped it at the first horizontal rule in the body, deleting the prose above it. What survived
// here is a bare `TODO`, so a written design was refused as "artifact is incomplete" and the
// author had no way to see why.
//
// Discriminates: on the unfixed binary this asserts `false` on a `true` — the artifact is refused.
#[test]
fn a_padded_closing_delimiter_does_not_delete_the_prose_above_a_body_horizontal_rule() {
    let content = "---\nchange: CHG-1\nartifact: design\n---  \n\n\
# Design\n\nThe retry budget is per-host, not per-request.\n\n---\n\nTODO\n";

    assert!(
        !artifact_content_is_incomplete(content),
        "written prose above a body horizontal rule must survive a padded closing delimiter"
    );
}

// Honest label: CHARACTERIZATION. It passes on the unfixed binary too — that is why it is here.
//
// #696 replaced this module's own `strip_yaml_frontmatter`, which had no BOM trim, so a
// BOM-prefixed artifact kept its frontmatter, the YAML counted as prose, and an artifact whose
// body was empty or only TODO passed the completeness gate. The replacement fixed that without
// claiming it and without a test. Undisclosed correct behaviour is still undisclosed; this is the
// record of it (#716).
#[test]
fn a_bom_prefixed_artifact_with_no_written_body_is_incomplete() {
    assert!(artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n"
    ));
    assert!(artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n\nTODO\n"
    ));
    // ...and a BOM must not make a WRITTEN artifact read as incomplete either.
    assert!(!artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n\nThe retry budget is per-host.\n"
    ));
}

// Honest label: CHARACTERIZATION of a KNOWN RESIDUAL, and it passes on the unfixed binary. It
// asserts a WRONG verdict on purpose, because the alternative is worse.
//
// `----` is a legal Markdown thematic break. Treating it as an opening delimiter would make the
// stripper scan forward to the next rule and return a body cut at it, which is the failure the
// canonical reader exists to prevent (#697, #699, #705) — so an artifact that is nothing but
// frontmatter opened with `----` still reads as complete here. Deriving the gate from the
// generated scaffold instead does not close this either: a file with a mangled opener no longer
// equals the scaffold, so it would read as written for the same reason.
//
// If this test ever fails, the hole closed — check that it closed for a defensible reason and
// delete the test, do not "restore" it.
#[test]
fn a_four_dash_opener_still_hides_an_empty_artifact_from_the_gate() {
    assert!(!artifact_content_is_incomplete(
        "----\nchange: CHG-1\nartifact: design\n---\n"
    ));
}

#[test]
fn validate_artifacts_rejects_hash_todo_body() {
    let temp = TempDir::new().unwrap();
    let record = completed_no_spec_record(temp.path());
    let dir = change_dir(temp.path(), &record.id);
    for artifact in &record.selected_artifacts {
        fs::write(
            dir.join(artifact.file_name()),
            format!(
                "---\nchange: {}\nartifact: {}\n---\n\n# TODO\n",
                record.id,
                artifact.file_name().trim_end_matches(".md")
            ),
        )
        .unwrap();
    }
    let error = validate_artifacts(temp.path(), &record).unwrap_err();
    assert!(
        error.contains("artifact is incomplete"),
        "expected incomplete artifact, got {error}"
    );
}

/// Build a repo whose committed ledger is `committed` and whose working tree
/// ledger is `working`, which is the divergence #533 commits backwards.
fn ledger_divergence_fixture(root: &Path, committed: u64, working: u64) {
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let write = |sequence: u64| {
        fs::write(
            root.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };
    write(committed);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "committed high-water mark"]);
    write(working);
}

/// Build a repo with an `origin` whose main carries `origin_seq`, and a local
/// branch whose ledger is `local_seq`. `diverged` decides which shape:
///
/// - `false` — the branch is honestly BEHIND: it was cut from origin at
///   `local_seq` and simply has not caught up. Nothing is wrong with it.
/// - `true` — the branch was cut from origin's tip and then REWROTE the ledger
///   downwards, which is the #533 regression.
///
/// On disk these two are indistinguishable: both show a ledger below origin with
/// no higher workspaces present. Only the history separates them, which is why
/// the gate compares against the merge-base and not against origin.
fn remote_divergence_fixture(root: &Path, origin_seq: u64, local_seq: u64, diverged: bool) {
    let git = |dir: &Path, args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .unwrap()
                .success()
        );
    };
    let origin = root.join("origin.git");
    let work = root.join("work");
    fs::create_dir_all(&work).unwrap();
    git(root, &["init", "--bare", "origin.git"]);
    git(root, &["init", "-b", "main", "work"]);
    git(&work, &["config", "user.email", "test@example.com"]);
    git(&work, &["config", "user.name", "Test"]);
    git(
        &work,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    fs::create_dir_all(work.join(".specsync")).unwrap();
    let write = |dir: &Path, sequence: u64| {
        fs::write(
            dir.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };

    // Commit the branch point at `local_seq`.
    write(&work, local_seq);
    fs::write(work.join("README.md"), "base\n").unwrap();
    git(&work, &["add", "."]);
    git(&work, &["commit", "-m", "branch point"]);

    if diverged {
        // Origin advances FIRST, so the branch point is origin's tip; the local
        // commit below then lowers the ledger relative to where it diverged.
        write(&work, origin_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "origin high-water"]);
        git(&work, &["push", "origin", "main"]);
        write(&work, local_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "hand-regressed ledger"]);
    } else {
        // Push the branch point, then advance origin on a throwaway branch so
        // local's HEAD stays an ancestor of origin/main — honestly behind.
        git(&work, &["push", "origin", "main"]);
        git(&work, &["checkout", "-q", "-b", "ahead"]);
        write(&work, origin_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "origin advanced"]);
        git(&work, &["push", "origin", "ahead:main"]);
        git(&work, &["checkout", "-q", "main"]);
        write(&work, local_seq);
    }
    git(&work, &["fetch", "-q", "origin"]);
}

/// A branch that is merely BEHIND the default branch must not be refused (#533
/// regression).
///
/// The first attempt at the read-side gate compared the ledger against
/// `origin/main` directly. Every unrebased branch trips that: its ledger is
/// older than origin's and perfectly consistent with its own history. `change
/// new` returned exit 1 and told the user to `git checkout origin/HEAD --` a
/// file that was not corrupt.
#[test]
fn a_branch_merely_behind_the_default_branch_is_not_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    remote_divergence_fixture(root, 4, 2, false);
    let work = root.join("work");

    let result = validate_change_sequences(&work);
    assert!(
        result.is_ok(),
        "a branch that is behind origin has an older ledger by definition; \
refusing it punishes the ordinary state of every unrebased branch: {result:?}"
    );
}

/// The vacuity control for the test above: a branch that actually rewrote the
/// ledger downwards must still be refused, so "stop comparing against the
/// remote" cannot be satisfied by removing the gate.
#[test]
fn a_branch_that_lowered_the_ledger_after_diverging_is_still_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    remote_divergence_fixture(root, 4, 2, true);
    let work = root.join("work");

    let error = validate_change_sequences(&work).expect_err(
        "a ledger rewritten below the mark this branch diverged from is the #533 regression",
    );
    assert!(
        error.contains("CHG-0004"),
        "the error must name the mark that was lost, not merely say something is wrong: {error}"
    );
}

/// A branch that RAISED the ledger and then rewrote it downwards is caught,
/// even though it never fell below the mark it diverged at.
///
/// This is the case that decided the oracle. A merge-base comparison acquits
/// it: the branch diverged at 2, raised to 9, rewrote to 3, and 3 is still
/// above the divergence point. Only the branch's own recorded history — which
/// contains the 9 — convicts. The ledger is not a distance from the default
/// branch; it is a high-water mark this branch is accountable for.
#[test]
fn a_branch_that_raised_then_rewrote_the_ledger_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    let write = |sequence: u64| {
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };

    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write(2);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "branch point at 2"]);
    write(9);
    git(&["add", "."]);
    git(&["commit", "-m", "raised to 9"]);
    write(3);
    git(&["add", "."]);
    git(&["commit", "-m", "rewrote down to 3"]);

    let error = validate_change_sequences(root).expect_err(
        "a ledger below the highest mark this branch itself recorded is the #533 regression, \
even when it is still above where the branch diverged",
    );
    assert!(
        error.contains("CHG-0009"),
        "the error must name the mark that was lost: {error}"
    );
}

/// A ledger that went stale while the branch sat must not be committed backwards
/// (#533).
///
/// `change new` writes the ledger into the working tree only. Nothing commits it
/// until a later lifecycle step runs `git add -A`, so a value written days
/// earlier — correct when written — is staged over a higher mark the branch has
/// since caught up to. The allocation-time floor cannot help: the value did not
/// start wrong, it went stale.
#[test]
fn a_stale_sequence_ledger_is_raised_to_the_committed_mark_before_staging() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 3, 1);

    let raised = floor_sequence_ledger_to_committed(root).unwrap();
    assert_eq!(
        raised,
        Some((1, 3)),
        "the caller must be told what was raised and from where, so it can disclose it"
    );
    let now = load_change_sequence_ledger(root).unwrap().unwrap();
    assert_eq!(now.sequence, 3, "the high-water mark must not regress");
    assert_eq!(now.id, "CHG-0003-fixture");
}

/// The control that keeps the fix from becoming "always overwrite the ledger".
///
/// A working tree ahead of the committed mark is the ordinary case — that is
/// exactly what `change new` produces — and raising it would destroy the claim
/// the author just made. Without this, a fix that unconditionally restored the
/// committed value would pass the test above.
#[test]
fn a_sequence_ledger_ahead_of_the_committed_mark_is_left_alone() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 3, 7);

    assert_eq!(
        floor_sequence_ledger_to_committed(root).unwrap(),
        None,
        "nothing was raised, so nothing may be reported"
    );
    let now = load_change_sequence_ledger(root).unwrap().unwrap();
    assert_eq!(now.sequence, 7, "the author's newer claim must survive");
    assert_eq!(now.id, "CHG-0007-fixture");
}

/// Equal marks are not a divergence and must not be reported as one.
#[test]
fn a_sequence_ledger_equal_to_the_committed_mark_is_not_reported() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 5, 5);
    assert_eq!(floor_sequence_ledger_to_committed(root).unwrap(), None);
    assert_eq!(
        load_change_sequence_ledger(root).unwrap().unwrap().sequence,
        5
    );
}

#[test]
fn definition_digest_caps_canonical_bytes_not_larger_checkout_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join(".gitattributes"), "*.md text eol=crlf\n").unwrap();
    let record = completed_no_spec_record(root);
    let context = change_dir(root, &record.id).join("context.md");
    fs::write(
        &context,
        b"x\n".repeat(MAX_CHANGE_ARTIFACT_BYTES as usize / 2),
    )
    .unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track definition"]);
    fs::remove_file(&context).unwrap();
    quiet_git(
        root,
        &[
            "checkout",
            "HEAD",
            "--",
            &format!("{CHANGES_PATH}/{}/context.md", record.id),
        ],
    );

    assert!(fs::metadata(&context).unwrap().len() > MAX_CHANGE_ARTIFACT_BYTES);
    assert!(definition_digest(root, &record).is_ok());
}

#[test]
fn definition_digest_rejects_oversized_canonical_sparse_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let record = completed_no_spec_record(root);
    let context_relative = format!("{CHANGES_PATH}/{}/context.md", record.id);
    let context = root.join(&context_relative);
    fs::write(&context, vec![b'x'; MAX_CHANGE_ARTIFACT_BYTES as usize + 1]).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track oversized definition"]);
    quiet_git(
        root,
        &["update-index", "--skip-worktree", &context_relative],
    );
    fs::remove_file(context).unwrap();

    let error = definition_digest(root, &record).unwrap_err();
    assert!(error.contains("exceeds") && error.contains("byte limit"));
}

#[test]
fn non_git_policy_disables_only_changed_path_coverage() {
    let temp = TempDir::new().unwrap();
    write_lifecycle_test_policy(temp.path());
    assert!(
        !load_policy(temp.path())
            .unwrap()
            .require_change_for_meaningful_files
    );
    let report = check_project(temp.path());
    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.contains("changed paths"))
    );
}

#[test]
fn committed_policy_cannot_be_disabled_or_deleted_locally() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.require_change_for_meaningful_files = true;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::write(root.join(".specsync/version"), SDD_VERSION).unwrap();
    git(&["add", ".specsync/sdd.json", ".specsync/version"]);
    git(&["commit", "-m", "enable sdd"]);

    let mut policy = load_policy(root).unwrap();
    policy.enabled = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("changed without")),
        "unexpected report: {report:?}"
    );

    policy.enabled = true;
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );

    policy.require_change_for_meaningful_files = true;
    policy.meaningful_paths.clear();
    policy.ignored_paths.push(POLICY_PATH.into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );

    fs::remove_file(root.join(POLICY_PATH)).unwrap();
    fs::remove_file(root.join(".specsync/version")).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );
}

#[test]
fn clean_initial_commit_needs_no_changed_path_coverage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "clean\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "initial"]);
    assert!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn clean_feature_branch_still_requires_changed_path_coverage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"]
    );
}

fn minimal_archived_record(id: &str, affected_paths: Vec<String>) -> ChangeRecord {
    ChangeRecord {
        schema_version: 1,
        workflow_version: 2,
        workflow_origin_version: Some(2),
        id: id.into(),
        slug: "archive".into(),
        title: "Archive fixture".into(),
        description: "fixture".into(),
        kind: ChangeKind::Feature,
        state: ChangeState::Archived,
        canonical_applied: true,
        correction_count: 0,
        base_commit: None,
        created_at: 1,
        updated_at: 1,
        affected_specs: Vec::new(),
        affected_paths,
        no_spec_change: true,
        no_spec_change_rationale: Some("fixture".into()),
        acceptance_criteria: vec!["fixture".into()],
        selected_artifacts: Vec::new(),
        dependencies: Vec::new(),
        supersedes: Vec::new(),
        acceptance_owner_corrections: Vec::new(),
        legacy_archive_baseline_digest: None,
        answers: BTreeMap::new(),
    }
}

#[test]
fn same_pr_archived_change_covers_delivery_paths_with_zero_actives() {
    // Mirrors product Lifecycle gate after `change ship` on the same PR:
    // product paths remain in the base...HEAD delivery, actives are empty,
    // but the archive package is on the tip and must still cover.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "product tip"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"]
    );

    let archived = minimal_archived_record("CHG-0001-same-pr-archive", vec!["src/".into()]);
    let archive_dir = root
        .join(ARCHIVE_PATH)
        .join("2026-08-07-CHG-0001-same-pr-archive");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::write(
        archive_dir.join("state.json"),
        serde_json::to_string_pretty(&archived).unwrap(),
    )
    .unwrap();
    git(&["add", ARCHIVE_PATH]);
    git(&["commit", "-m", "archive tip"]);

    let uncovered = uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap();
    assert!(
        uncovered.is_empty(),
        "delivery archive must cover product paths with zero actives; got {uncovered:?}"
    );
}

#[test]
fn historical_archive_not_in_delivery_does_not_cover_unrelated_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    // Historical archive already on main — not part of this feature delivery.
    let historical = minimal_archived_record("CHG-0000-old", vec!["src/".into()]);
    let archive_dir = root.join(ARCHIVE_PATH).join("2026-01-01-CHG-0000-old");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::write(
        archive_dir.join("state.json"),
        serde_json::to_string_pretty(&historical).unwrap(),
    )
    .unwrap();
    git(&["add", "README.md", ARCHIVE_PATH]);
    git(&["commit", "-m", "base with old archive"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "uncovered feature"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"],
        "archives outside the delivery must not cover new product paths"
    );
}

#[test]
fn approved_change_does_not_cover_delivery_paths_until_started() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);
    let mut record = completed_record(root);
    // The ledger used to appear in the working tree because `change new` wrote it. Nothing
    // writes it now, so the fixture puts it there: the subject of this test is a *changed*
    // protected path that an approved change scopes, and it needs one to be changed.
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(SEQUENCE_PATH), "{}\n").unwrap();
    record.affected_paths = vec!["src/".into(), SEQUENCE_PATH.into(), POLICY_PATH.into()];
    record.state = ChangeState::Approved;
    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[record.clone()]).unwrap(),
        vec![
            SEQUENCE_PATH.to_string(),
            POLICY_PATH.to_string(),
            "src/lib.rs".into(),
        ]
    );
    record.state = ChangeState::Implementing;
    assert!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[record])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn local_coverage_unions_staged_unstaged_and_untracked_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "base\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::write(root.join("src/lib.rs"), "unstaged\n").unwrap();
    fs::write(root.join("src/staged.rs"), "staged\n").unwrap();
    git(&["add", "src/staged.rs"]);
    fs::write(root.join("src/untracked.rs"), "untracked\n").unwrap();
    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs", "src/staged.rs", "src/untracked.rs"]
    );
}

#[test]
fn accepted_changes_require_matching_closing_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        definition_digest(root, &record).unwrap(),
        None,
    )
    .unwrap();
    record.state = ChangeState::Accepted;
    save_change(root, &record).unwrap();
    let verification = VerificationRecord {
        timestamp: now(),
        commit: None,
        contract_digest: definition_digest(root, &record).unwrap(),
        execution_digest: None,
        workspace_digest: "workspace".into(),
        acceptance_input_digest: None,
        acceptance_manifest: None,
        semantic_succession: None,
        passed: true,
        commands: Vec::new(),
        requirement_ids: Vec::new(),
    };
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &verification,
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_err());
    append_approval(
        root,
        &record,
        "acceptance",
        Some("Reviewer".into()),
        closing_digest(&record, &verification),
        None,
    )
    .unwrap();
    let error = ensure_closing_approval_valid(root, &record).unwrap_err();
    assert!(error.contains("missing current delivery-input evidence"));
    let mut verification = verification;
    verification.acceptance_input_digest =
        Some(acceptance_input_digest(root, &record, &[]).unwrap());
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &verification,
    )
    .unwrap();
    append_approval(
        root,
        &record,
        "acceptance",
        Some("Reviewer".into()),
        closing_digest(&record, &verification),
        Some("Reapprove additive delivery-input evidence".into()),
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.approvals.last_mut().unwrap().digest = "tampered".into();
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_err());
}

#[test]
fn working_tree_changes_invalidate_verification() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(
        error.contains("verification project-input digest is stale"),
        "{error}"
    );
}

#[test]
fn stale_accepted_change_reopens_with_audited_evidence_and_reaccepts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let prior_verification = load_verification(root, &record).unwrap();
    let prior_ledger = load_approvals(root, &record).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let stale_report = check_project(root);
    assert!(stale_report.errors.iter().any(|error| {
        error.contains("accepted change verification is stale for current delivery inputs")
    }));
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!(
            "run `specsync change reopen {} --actor <name> --reason <reason>`",
            record.id
        )
    );

    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Review fixes changed scoped delivery inputs".into(),
    )
    .unwrap();
    record = reopened.change;
    assert_eq!(record.state, ChangeState::Verifying);
    assert!(record.canonical_applied);
    assert_eq!(reopened.audit.superseded_approval.actor, "Closer");
    assert_eq!(
        reopened.audit.prior_verification.contract_digest,
        prior_verification.contract_digest
    );
    let reopened_ledger = load_approvals(root, &record).unwrap();
    assert_eq!(
        reopened_ledger.approvals.len(),
        prior_ledger.approvals.len()
    );
    assert_eq!(reopened_ledger.reopenings.len(), 1);
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains("verification evidence is stale for the current commit or contract")
    }));

    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(check_project(root).errors.is_empty());
    let final_ledger = load_approvals(root, &record).unwrap();
    assert_eq!(
        final_ledger.approvals.len(),
        prior_ledger.approvals.len() + 1
    );
    assert_eq!(final_ledger.reopenings.len(), 1);
    assert_eq!(
        final_ledger.reopenings[0]
            .prior_verification
            .contract_digest,
        prior_verification.contract_digest
    );
}

// Verifies reopen recovery when verification.json no longer matches the closing
// approval (tip re-verify drift) but attempt history still authenticates it.
#[test]
fn reopen_binds_historical_verification_when_tip_no_longer_matches_closing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let accepted_verification = load_verification(root, &record).unwrap();
    let closing = latest_terminal_approval(&load_approvals(root, &record).unwrap())
        .cloned()
        .expect("closing approval");
    assert_eq!(
        closing.digest,
        closing_digest(&record, &accepted_verification)
    );

    // Drift the tip: rewrite verification.json to a fresh pre-accept shape that no
    // longer carries the acceptance digest the closing approval signed.
    let mut drifted = accepted_verification.clone();
    drifted.acceptance_input_digest = None;
    drifted.acceptance_manifest = None;
    drifted.semantic_succession = None;
    drifted.timestamp = now();
    record_verification_attempt(root, &record, &drifted).unwrap();
    assert_ne!(
        closing.digest,
        closing_digest(&record, &load_verification(root, &record).unwrap())
    );

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();

    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Tip verification drifted after accept; delivery inputs also changed".into(),
    )
    .expect("reopen must bind the historical acceptance-bound verification");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.prior_verification.acceptance_input_digest,
        accepted_verification.acceptance_input_digest
    );

    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(check_project(root).errors.is_empty());
    let closing_after = latest_terminal_approval(&load_approvals(root, &record).unwrap())
        .cloned()
        .expect("new closing approval");
    assert_eq!(
        closing_after.digest,
        closing_digest(&record, &load_verification(root, &record).unwrap())
    );
}

// Verifies workflow-v2 same-PR finalize after reopen writes a terminal finalization
// approval that a later reopen can supersede.
#[test]
fn workflow_v2_reopen_after_finalization_accept_then_finalize_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);

    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    assert!(record.workflow_version >= 2);
    record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();

    // Accept via finalization gate without archiving yet (interrupted finalize shape).
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    let ledger = load_approvals(root, &record).unwrap();
    assert!(
        latest_terminal_approval(&ledger).is_some_and(|a| a.gate == "finalization"),
        "finalization must leave a terminal ledger approval"
    );

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "delivery input changed after finalization accept".into(),
    )
    .expect("v2 reopen must supersede finalization closing approval");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(reopened.audit.superseded_approval.gate, "finalization");

    git(&["add", "."]);
    git(&["commit", "-m", "fix after reopen"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let path = finalize_change(root, &record.id).expect("finalize after reopen must archive");
    assert!(path.join("finalization.json").exists());
    assert!(!change_dir(root, &record.id).exists());
}

#[test]
fn accepted_change_can_refresh_stale_definition_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);

    // Mutate a selected artifact so the definition digest drifts while accepted.
    let context = change_dir(root, &record.id).join("context.md");
    let mut body = fs::read_to_string(&context).unwrap();
    body.push_str("\n\nRefreshed context while accepted.\n");
    fs::write(&context, body).unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_err());

    record = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect("accepted records must re-approve a stale definition");
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn legacy_workflow_finalize_refuses_and_names_accept_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    assert_eq!(record.workflow_version, 1);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let error = finalize_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("legacy workflow") && error.contains("change accept"),
        "{error}"
    );
}

// Verifies REQ-change-034.
#[test]
fn stale_accepted_change_error_names_uncovered_input_and_reopen_remediation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: delivery input `src/lib.rs` (owner `change`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen {}` to re-verify the accepted change",
        record.id, record.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| *error == expected)
    );
}

/// A legacy accepted change whose `auth` inputs have since changed, and the later accepted
/// change that supersedes every one of them. Both acceptances are committed and recorded on the
/// remote default branch, so `check_project` is clean until one of the two is disturbed.
fn accepted_change_with_covering_successor(root: &Path) -> (ChangeRecord, ChangeRecord) {
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut predecessor = completed_section_only_record(root, delta);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let predecessor_manifest = load_verification(root, &predecessor)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let mut successor = completed_section_only_record(root, delta);
    successor.affected_paths.extend(
        predecessor_manifest
            .entries
            .iter()
            .filter(|entry| {
                entry.owners.iter().any(|owner| owner == "auth")
                    && entry.path != "specs/auth/requirements.md"
            })
            .map(|entry| entry.path.clone()),
    );
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for entry in predecessor_manifest.entries.iter().filter(|entry| {
        entry.owners.iter().any(|owner| owner == "auth")
            && entry.path != "specs/auth/requirements.md"
    }) {
        successor = add_supersedes_obligation(
            root,
            &successor.id,
            &predecessor.id,
            &entry.path,
            "auth",
            &entry.entry_digest,
        )
        .unwrap();
    }
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    successor = start_implementation(root, &successor.id).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
    verify_change(root, &successor.id).unwrap();
    successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept semantic successor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    assert!(check_project(root).errors.is_empty());
    (predecessor, successor)
}

// Verifies REQ-change-036.
#[test]
fn stale_accepted_change_error_names_covering_successor_with_stale_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, successor) = accepted_change_with_covering_successor(root);

    fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: delivery input `specs/auth/auth.spec.md` (owner `auth`) changed after acceptance; successor `{}` was rejected: its own delivery-input evidence is stale: delivery input `src/auth.rs` (owner `auth`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen {}` to re-verify the accepted change; verify and accept a covering successor, or run `specsync change reopen {}` to re-verify the accepted change",
        predecessor.id, successor.id, successor.id, predecessor.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
}

/// The successor walk must say which successor it refused and why (#685). Its authentication
/// failures used to be `Err(_) => continue`, so the diagnostic claimed no successor existed.
// Verifies REQ-change-036.
#[test]
fn stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, successor) = accepted_change_with_covering_successor(root);

    let mut verification = load_verification(root, &successor).unwrap();
    verification.passed = false;
    write_json(
        &change_dir(root, &successor.id).join("verification.json"),
        &verification,
    )
    .unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: delivery input `specs/auth/auth.spec.md` (owner `auth`) changed after acceptance; successor `{}` was rejected: its accepted evidence did not authenticate: accepted change has failed verification evidence; verify and accept a covering successor, or run `specsync change reopen {}` to re-verify the accepted change",
        predecessor.id, successor.id, predecessor.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
}

/// The archive post-move preflight must accept a workflow-v2 successor that supersedes a legacy
/// accepted change's inputs -- during that successor's OWN finalize.
///
/// Field reproduction: CorvidLabs/swift-algorand `chore/specsync-6`, where a v2 change superseding
/// six inputs of legacy `CHG-0001` was rated `exact` by `change audit` and then refused by
/// `finalize` with "no accepted or archived successor change covers it". The only possible
/// successor was the package being closed, and the preflight judged it by a history the closing
/// commit had not yet written.
// Verifies REQ-change-024 and REQ-change-036.
#[test]
fn finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut predecessor = completed_section_only_record(root, delta);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    assert_eq!(predecessor.workflow_version, 1);
    git(&["add", "."]);
    git(&["commit", "-m", "accept legacy predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let obligations: Vec<AcceptanceInputEntryV1> = load_verification(root, &predecessor)
        .unwrap()
        .acceptance_manifest
        .unwrap()
        .entries
        .into_iter()
        .filter(|entry| {
            entry.owners.iter().any(|owner| owner == "auth")
                && entry.path != "specs/auth/requirements.md"
        })
        .collect();

    let mut successor = current_workflow_record(root, completed_current_record(root));
    for artifact in &successor.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete the successor.\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
        };
        fs::write(
            change_dir(root, &successor.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    fs::write(
        delta_path(root, &successor, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the successor.\n",
    )
    .unwrap();
    successor
        .affected_paths
        .extend(obligations.iter().map(|entry| entry.path.clone()));
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for entry in &obligations {
        successor = add_supersedes_obligation(
            root,
            &successor.id,
            &predecessor.id,
            &entry.path,
            "auth",
            &entry.entry_digest,
        )
        .unwrap();
    }
    successor = approve_definition(root, &successor.id, Some("Scope owner".into()), None).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
    check_change(root, Some(&successor.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement successor"]);
    check_change(root, Some(&successor.id)).unwrap();
    record_scoped_review(root, &successor.id, "Independent reviewer".into()).unwrap();

    let archived = finalize_change(root, &successor.id).unwrap();
    assert!(
        archived.starts_with(root.join(ARCHIVE_PATH)),
        "{}",
        archived.display()
    );
    assert_eq!(
        load_change(root, &successor.id).unwrap().state,
        ChangeState::Archived
    );
    // The legacy predecessor is successor-covered before the archive commit is made, and after --
    // on the full walk and on the active-only audit. The audit evaluates no archive, but the
    // archived successor must still be in its candidate universe, or the predecessor reads as
    // uncovered by the very change that just finalized over it (the swift-algorand field
    // finding, second round).
    let successor_covers = |root: &Path| {
        let report = check_project(root);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let audit = audit_project(root);
        assert!(audit.errors.is_empty(), "{:?}", audit.errors);
        let evidence = audit
            .terminal_evidence
            .iter()
            .find(|evidence| evidence.id == predecessor.id)
            .unwrap_or_else(|| panic!("{:?}", audit.terminal_evidence));
        assert_eq!(
            evidence.evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            evidence.evidence.reason
        );
    };
    successor_covers(root);
    git(&["add", "."]);
    git(&["commit", "-m", "Finalize successor"]);
    successor_covers(root);

    // Disturb the successor's own inputs. The predecessor is stale again, and the diagnostic
    // names the successor it refused and why -- and does not offer the legacy reopen, which would
    // replay the predecessor's canonical delta over the successor's materialization.
    fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
    let report = check_project(root);
    let error = report
        .errors
        .iter()
        .find(|error| error.starts_with(&format!("{}: ", predecessor.id)))
        .unwrap_or_else(|| panic!("{:?}", report.errors));
    assert!(
        error.contains(&format!(
            "successor `{}` was rejected: its own delivery-input evidence is stale: delivery input `src/auth.rs` (owner `auth`) changed after acceptance",
            successor.id
        )),
        "{error}"
    );
    assert!(
        error.contains(&format!(
            "bring a covering successor to `specsync change finalize` (`specsync change status {}` names its next step) rather than reopening `{}`",
            successor.id, predecessor.id
        )),
        "{error}"
    );
    assert!(
        !error.contains(&format!("run `specsync change reopen {}`", predecessor.id)),
        "{error}"
    );
    // The audit says the same thing as the full walk: it refused the same successor for the same
    // reason, rather than never having seen it.
    let audit = audit_project(root);
    assert!(audit.errors.contains(error), "{:?}", audit.errors);
}

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success(),
        "git command failed: {args:?}"
    );
}

/// A repository bootstrapped by a whole-tree legacy change, the swift-algorand shape: the
/// bootstrap declares the test tree and the package manifest beside its module's source, and
/// is accepted and committed before any `owns` configuration exists, so its signed manifest
/// carries every non-spec path under a reserved exact owner. Returns the accepted bootstrap
/// and its signed manifest.
fn exact_only_bootstrap_fixture(root: &Path) -> (ChangeRecord, AcceptanceManifestV1) {
    run_git(root, &["init", "-b", "main"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    run_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    fs::create_dir_all(root.join("tests/auth")).unwrap();
    fs::write(root.join("tests/auth/legacy.rs"), "// XCTest-era test.\n").unwrap();
    fs::write(
        root.join("tests/auth/deprecated.rs"),
        "// Exercises a deprecated trapping API.\n",
    )
    .unwrap();
    fs::write(root.join("Package.swift"), "// swift-tools-version: 6.0\n").unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "base"]);

    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut bootstrap = completed_section_only_record(root, delta);
    bootstrap
        .affected_paths
        .extend(["tests/auth".to_string(), "Package.swift".to_string()]);
    bootstrap.affected_paths.sort();
    save_change(root, &bootstrap).unwrap();
    write_change_markdown(root, &bootstrap).unwrap();
    bootstrap = approve_definition(root, &bootstrap.id, Some("Reviewer".into()), None).unwrap();
    bootstrap = start_implementation(root, &bootstrap.id).unwrap();
    verify_change(root, &bootstrap.id).unwrap();
    bootstrap = accept_change(root, &bootstrap.id, Some("Closer".into()), None).unwrap();
    assert_eq!(bootstrap.workflow_version, 1);
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "accept bootstrap"]);
    run_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let signed = load_verification(root, &bootstrap)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let owners = |path: &str| {
        signed
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .unwrap_or_else(|| panic!("{path}: {:?}", signed.entries))
            .owners
            .clone()
    };
    assert_eq!(owners("src/auth.rs"), vec!["auth".to_string()]);
    assert_eq!(
        owners("tests/auth/legacy.rs"),
        vec![EXACT_TEST_OWNER.to_string()]
    );
    assert_eq!(
        owners("tests/auth/deprecated.rs"),
        vec![EXACT_TEST_OWNER.to_string()]
    );
    assert_eq!(
        owners("Package.swift"),
        vec![EXACT_DELIVERY_OWNER.to_string()]
    );
    (bootstrap, signed)
}

/// A workflow-v2 successor of the bootstrap declaring the same test tree and package manifest
/// plus `extra_paths`, with complete artifacts and an `auth` delta, ready to adopt obligations.
fn successor_over_exact_only_bootstrap(root: &Path, extra_paths: &[&str]) -> ChangeRecord {
    let mut successor = current_workflow_record(root, completed_current_record(root));
    for artifact in &successor.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete the successor.\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
        };
        fs::write(
            change_dir(root, &successor.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    fs::write(
        delta_path(root, &successor, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the successor.\n",
    )
    .unwrap();
    successor
        .affected_paths
        .extend(["tests/auth".to_string(), "Package.swift".to_string()]);
    successor
        .affected_paths
        .extend(extra_paths.iter().map(|path| path.to_string()));
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    successor
}

fn manifest_entry<'a>(
    manifest: &'a AcceptanceManifestV1,
    path: &str,
) -> &'a AcceptanceInputEntryV1 {
    manifest
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .unwrap_or_else(|| panic!("{path}: {:?}", manifest.entries))
}

// Verifies REQ-change-050 with REQ-change-020 and REQ-change-024.
//
// Field shape: CorvidLabs/swift-algorand, bootstrapped by a whole-tree governance change
// (`CHG-0001` declared `Sources/`, `Tests/`, `Package.swift`, `.github/`, ...). Its acceptance
// manifest gave `algorand` only the paths its spec's `files:` listed; every legacy XCTest file,
// `Package.swift`, and the DocC catalog were signed `@exact:test` or `@exact:delivery`. An
// exact-only input could not be superseded (`change supersede --spec algorand` refused the
// module as "not a successor-eligible signed owner"), `@exact:delivery` is not a module, and the
// preflight's only remedy for a changed one was an audited reopen of the bootstrap -- which
// replays its canonical delta over every successor's materialization. So a Swift Testing port
// of the legacy tests, removal of the deprecated APIs they exercised, and a DocC gate in
// `Package.swift` (PR #20's carve-out) were frozen for every later change.
//
// With `[modules."auth"] owns = ["tests/auth", "Package.swift"]`, the successor's own entries
// for those paths -- the directory entry included -- are signed `auth`, the frozen `@exact:*`
// predecessor entries are successor-eligible for the module that owns the paths now, and
// finalize archives the successor with the bootstrap successor-covered on both surfaces, before
// and after the archive commit. No `correct-owner`, no reopen, and `check` demands no spec
// coverage for the owned paths because they are not source mappings.
#[test]
fn configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change()
 {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (bootstrap, signed) = exact_only_bootstrap_fixture(root);
    let directory = manifest_entry(&signed, "tests/auth");
    assert_eq!(directory.kind, AcceptanceInputKind::NonFile);
    assert_eq!(directory.owners, vec![EXACT_TEST_OWNER.to_string()]);

    // Before the configuration grants `auth` the paths, an exact-only entry refuses the module
    // -- the field refusal, now naming the remedy -- and the refusal persists nothing.
    let mut successor = successor_over_exact_only_bootstrap(root, &["specs/auth/auth.spec.md"]);
    let refused = add_supersedes_obligation(
        root,
        &successor.id,
        &bootstrap.id,
        "tests/auth/legacy.rs",
        "auth",
        &manifest_entry(&signed, "tests/auth/legacy.rs").entry_digest,
    )
    .unwrap_err();
    assert_eq!(
        refused,
        "module `auth` does not currently own predecessor path `tests/auth/legacy.rs` (signed `@exact:test`); grant it with `owns` under `[modules.\"auth\"]` in `.specsync/config.toml`"
    );
    assert!(
        load_change(root, &successor.id)
            .unwrap()
            .supersedes
            .is_empty()
    );

    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.\"auth\"]\nowns = [\"tests/auth\", \"Package.swift\"]\n",
    )
    .unwrap();
    let obligations = [
        "src/auth.rs",
        "specs/auth/auth.spec.md",
        "tests/auth/legacy.rs",
        "tests/auth/deprecated.rs",
        "Package.swift",
    ];
    for path in obligations {
        successor = add_supersedes_obligation(
            root,
            &successor.id,
            &bootstrap.id,
            path,
            "auth",
            &manifest_entry(&signed, path).entry_digest,
        )
        .unwrap();
    }
    successor = approve_definition(root, &successor.id, Some("Scope owner".into()), None).unwrap();
    // Remove the deprecated API, port one legacy test, delete the one that exercised the
    // removed API, and gate a dependency in the package manifest.
    fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
    fs::write(
        root.join("tests/auth/legacy.rs"),
        "// Swift Testing port.\n",
    )
    .unwrap();
    fs::remove_file(root.join("tests/auth/deprecated.rs")).unwrap();
    fs::write(
        root.join("Package.swift"),
        "// swift-tools-version: 6.0\n// DocC plugin gated behind an environment flag.\n",
    )
    .unwrap();
    check_change(root, Some(&successor.id)).unwrap();
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "Implement successor"]);
    check_change(root, Some(&successor.id)).unwrap();
    record_scoped_review(root, &successor.id, "Independent reviewer".into()).unwrap();

    let archived = finalize_change(root, &successor.id).unwrap();
    assert!(
        archived.starts_with(root.join(ARCHIVE_PATH)),
        "{}",
        archived.display()
    );
    let successor = load_change(root, &successor.id).unwrap();
    assert_eq!(successor.state, ChangeState::Archived);

    // The successor signs every owned path under `auth`, the directory entry and the deleted
    // file included, and each succession tuple binds the frozen predecessor entry digest to the
    // module that owns the path now.
    let verification = load_verification(root, &successor).unwrap();
    let manifest = verification.acceptance_manifest.clone().unwrap();
    for path in ["tests/auth", "Package.swift"]
        .into_iter()
        .chain(obligations)
    {
        assert_eq!(
            manifest_entry(&manifest, path).owners,
            vec!["auth".to_string()],
            "{path}"
        );
    }
    assert_eq!(
        manifest_entry(&manifest, "tests/auth").kind,
        AcceptanceInputKind::NonFile
    );
    assert_eq!(
        manifest_entry(&manifest, "tests/auth/deprecated.rs").kind,
        AcceptanceInputKind::Missing
    );
    let tuples = verification.semantic_succession.unwrap().tuples;
    assert_eq!(tuples.len(), obligations.len(), "{tuples:?}");
    for path in obligations {
        let tuple = tuples
            .iter()
            .find(|tuple| tuple.path == path)
            .unwrap_or_else(|| panic!("{path}: {tuples:?}"));
        assert_eq!(tuple.predecessor_id, bootstrap.id);
        assert_eq!(tuple.module, "auth");
        assert_eq!(
            tuple.predecessor_entry_digest,
            manifest_entry(&signed, path).entry_digest
        );
        assert_eq!(
            tuple.successor_entry_digest,
            manifest_entry(&manifest, path).entry_digest
        );
    }

    // The bootstrap is successor-covered before the archive commit and after it, on the full
    // walk and on the active-only audit, without a reopen or an owner correction.
    let successor_covers = |root: &Path| {
        let report = check_project(root);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let audit = audit_project(root);
        assert!(audit.errors.is_empty(), "{:?}", audit.errors);
        let evidence = audit
            .terminal_evidence
            .iter()
            .find(|evidence| evidence.id == bootstrap.id)
            .unwrap_or_else(|| panic!("{:?}", audit.terminal_evidence));
        assert_eq!(
            evidence.evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            evidence.evidence.reason
        );
    };
    successor_covers(root);
    run_git(root, &["add", "."]);
    run_git(root, &["commit", "-m", "Finalize successor"]);
    successor_covers(root);

    // Disturbing an owned input after the archive stales the successor, so every exact-only
    // entry it covered is reported through it -- the first in path order is `Package.swift`,
    // with the frozen label as its owner -- naming the archived successor and its reason exactly
    // as a signed owner's successor would be named, and steering away from the reopen.
    fs::write(
        root.join("tests/auth/legacy.rs"),
        "// Edited after the archive.\n",
    )
    .unwrap();
    let report = check_project(root);
    let error = report
        .errors
        .iter()
        .find(|error| error.starts_with(&format!("{}: ", bootstrap.id)))
        .unwrap_or_else(|| panic!("{:?}", report.errors));
    assert!(
        error.contains(&format!(
            "delivery input `Package.swift` (owner `@exact:delivery`) changed after acceptance; successor `{}` was rejected: its own delivery-input evidence is stale: delivery input `tests/auth/legacy.rs` (owner `auth`) changed after acceptance",
            successor.id
        )),
        "{error}"
    );
    assert!(
        !error.contains(&format!("run `specsync change reopen {}`", bootstrap.id)),
        "{error}"
    );
}

// Verifies REQ-change-050.
// The configuration grants `auth` the test tree and nothing else. `Package.swift` stays
// `@exact:delivery` for the successor too, so the bootstrap's frozen entry for it has no
// module that owns it now, and the obligation is refused with the same message the field saw
// -- after the granted path has already been adopted, and without touching the record.
#[test]
fn supersede_refuses_an_exact_only_input_the_configuration_grants_no_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (bootstrap, signed) = exact_only_bootstrap_fixture(root);
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.\"auth\"]\nowns = [\"tests/auth\"]\n",
    )
    .unwrap();
    let successor = successor_over_exact_only_bootstrap(root, &[]);
    let successor = add_supersedes_obligation(
        root,
        &successor.id,
        &bootstrap.id,
        "tests/auth/legacy.rs",
        "auth",
        &manifest_entry(&signed, "tests/auth/legacy.rs").entry_digest,
    )
    .unwrap();
    let refused = add_supersedes_obligation(
        root,
        &successor.id,
        &bootstrap.id,
        "Package.swift",
        "auth",
        &manifest_entry(&signed, "Package.swift").entry_digest,
    )
    .unwrap_err();
    assert_eq!(
        refused,
        "module `auth` does not currently own predecessor path `Package.swift` (signed `@exact:delivery`); grant it with `owns` under `[modules.\"auth\"]` in `.specsync/config.toml`"
    );
    let persisted = load_change(root, &successor.id).unwrap();
    assert_eq!(persisted.supersedes, successor.supersedes);
    assert_eq!(persisted.supersedes.len(), 1);
    assert_eq!(persisted.supersedes[0].obligations.len(), 1);
    assert_eq!(
        persisted.supersedes[0].obligations[0].path,
        "tests/auth/legacy.rs"
    );

    // A signed module owner's claim stays with that module: the historical rule is unchanged
    // for an entry a module signed, whatever the configuration says now.
    let evidence = acceptance_owner_spec_evidence(root, &persisted).unwrap();
    assert_eq!(
        acceptance_input_owners(
            root,
            &persisted,
            "src/auth.rs",
            &[],
            &evidence,
            UnownedProductionSource::Reject,
        )
        .unwrap(),
        vec!["auth".to_string()]
    );
}

// Verifies REQ-change-050.
#[test]
fn configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nTests remain governed.\n",
    );
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[modules.\"auth\"]\nowns = [\"tests/auth\", \"Package.swift\", \".specsync\"]\n\n[modules.\"billing\"]\nowns = [\"tests/billing\"]\n",
    )
    .unwrap();
    let evidence = git_evidence(root, &BTreeSet::new()).unwrap();
    let owners = |path: &str| {
        acceptance_input_owners(
            root,
            &record,
            path,
            &[],
            &evidence,
            UnownedProductionSource::Reject,
        )
        .unwrap()
    };
    // A configured path is the declared module's input, whether it is a test file, the
    // directory entry itself, or root delivery metadata.
    assert_eq!(owners("tests/auth/legacy.rs"), vec!["auth".to_string()]);
    assert_eq!(owners("tests/auth"), vec!["auth".to_string()]);
    assert_eq!(owners("Package.swift"), vec!["auth".to_string()]);
    // The reserved classes stand wherever nothing declared owns the path: a module the change
    // does not declare, a test tree no module owns, and a mapped-but-unowned test.
    assert_eq!(
        owners("tests/billing/invoice.rs"),
        vec![EXACT_TEST_OWNER.to_string()]
    );
    assert_eq!(owners("tests/other.rs"), vec![EXACT_TEST_OWNER.to_string()]);
    assert_eq!(
        owners("fledge.toml"),
        vec![EXACT_DELIVERY_OWNER.to_string()]
    );
    // The lifecycle's own ledger is never configurable ownership.
    assert_eq!(
        owners(".specsync/config.toml"),
        vec![EXACT_DELIVERY_OWNER.to_string()]
    );
    assert_eq!(
        owners(".specsync/change-sequence.json"),
        vec![EXACT_DELIVERY_OWNER.to_string()]
    );
}

// Verifies REQ-change-034.
#[test]
fn stale_accepted_change_error_names_exact_only_input_and_audited_reopen() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "Initial review instructions.\n").unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "update review instructions".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Documentation-only review guidance".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Reviewers can follow the release workflow".into()];
    record.answers.insert("public_contract".into(), "no".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    persist_legacy_test_record(root, &mut record);
    write_change_markdown(root, &record).unwrap();
    for artifact in &record.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(root.join("README.md"), "Final review instructions.\n").unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: exact-only delivery input `README.md` changed after acceptance and requires an audited reopen; run `specsync change reopen {}` to re-verify the accepted change, or supersede it from a later change under a module granted the path by `owns` in `.specsync/config.toml`",
        record.id, record.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
}

// Verifies REQ-change-033.
// Canonical ownership is knowable the moment a path is declared, but was
// enforced only when building the acceptance manifest at finalize. A change
// declaring a path owned by a module it does not name therefore passed
// `approve` and every `check`, was reviewed, and only then failed — into a
// state with no exit, since `correct-owner` is scoped to already-applied
// changes and `reopen` takes only Accepted/Archived.
//
// Rejecting at approve costs two seconds instead of several verification
// passes and a reviewer's signature, and keeps the unrecoverable state
// unreachable.
#[test]
fn approve_rejects_a_declared_path_owned_by_an_undeclared_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();

    // `legacy` owns src/lib.rs but not src/orphan.rs. The change declares
    // `legacy` and touches both — the shape CHG-0081 was stuck in, where a
    // change edits a file belonging to a module it never declared.
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn owned() {}\n").unwrap();
    fs::write(root.join("src/orphan.rs"), "pub fn unowned() {}\n").unwrap();

    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    record.affected_paths = vec!["src/lib.rs".into(), "src/orphan.rs".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must reject a path no declared module owns");
    assert!(
        error.contains("src/orphan.rs") && error.to_lowercase().contains("own"),
        "error should name the unowned path and the ownership problem: {error}"
    );
    assert!(
        !error.contains("src/lib.rs"),
        "the owned path must not be reported as a problem: {error}"
    );
    assert!(
        error.contains("--spec") || error.contains("files:"),
        "error should say how to resolve it, not just that it failed: {error}"
    );
}

// A change that reaches Verifying without ever closing has no reopen event,
// because nothing ever closed it. Owner correction demanded one, so a change
// whose acceptance inputs touch a file owned by an undeclared module could
// neither finalize (ownership unresolved) nor be corrected (no reopen) nor be
// reopened (reopen takes only Accepted/Archived). Every exit was closed.
//
// The reopen proves definition match against closing verification. A
// never-closed change substitutes a live definition approval so the guided
// path stays reachable (weaker provenance, not audit-equivalent).
#[test]
fn never_closed_verifying_change_corrects_an_owner_without_a_reopen() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();

    // Deliberately no accept and no reopen: the change sits at Verifying,
    // exactly where a reviewed change waits for finalize. `check` materializes
    // the delta before verifying, so a change that reached this point has its
    // canonical application recorded — which is what the real stranded changes
    // look like, and what distinguishes this from a draft.
    record = load_change(root, &record.id).unwrap();
    record.canonical_applied = true;
    save_change(root, &record).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);

    let corrected = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "The definition omitted the canonical owner of an affected path".into(),
    )
    .expect("a never-closed verifying change has no reopen to reference");
    assert_eq!(corrected.acceptance_owner_corrections.len(), 1);
}

#[test]
fn reopened_change_adds_exact_canonical_owner_without_replaying_delivery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "The accepted source changed during release review".into(),
    )
    .unwrap()
    .change;
    record = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "The historical definition omitted the current canonical owner".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 1);
    assert_eq!(record.acceptance_owner_corrections[0].sequence, 1);
    let state_path = change_dir(root, &record.id).join("state.json");
    let corrected_state = fs::read(&state_path).unwrap();
    let duplicate = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "Duplicate owner".into(),
    )
    .unwrap_err();
    assert!(duplicate.contains("duplicate acceptance owner correction"));
    assert_eq!(fs::read(&state_path).unwrap(), corrected_state);

    let mut tampered = record.clone();
    tampered.acceptance_owner_corrections[0].sequence = 2;
    save_change(root, &tampered).unwrap();
    let error = validate_definition(root, &tampered).unwrap_err();
    assert!(error.contains("sequence is not contiguous"), "{error}");
    save_change(root, &record).unwrap();

    let mut broadened = record.clone();
    broadened
        .dependencies
        .push("CHG-9999-unapproved-scope".into());
    save_change(root, &broadened).unwrap();
    let error = ensure_reopened_definition_unchanged(root, &broadened).unwrap_err();
    assert!(error.contains("modified definition"), "{error}");
    save_change(root, &record).unwrap();

    let portable = TempDir::new().unwrap();
    let portable_root = portable.path();
    let portable_dir = change_dir(portable_root, &record.id);
    fs::create_dir_all(portable_dir.join("deltas")).unwrap();
    save_change(portable_root, &record).unwrap();
    for artifact in &record.selected_artifacts {
        fs::copy(
            change_dir(root, &record.id).join(artifact.file_name()),
            portable_dir.join(artifact.file_name()),
        )
        .unwrap();
    }
    assert_eq!(
        definition_digest(root, &record).unwrap(),
        definition_digest(portable_root, &record).unwrap()
    );
    assert!(
        ensure_definition_approval_valid(root, &record)
            .unwrap_err()
            .contains("definition approval is stale")
    );

    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let verification = load_verification(root, &record).unwrap();
    let source = verification
        .acceptance_manifest
        .unwrap()
        .entries
        .into_iter()
        .find(|entry| entry.path == "src/lib.rs")
        .unwrap();
    assert_eq!(source.owners, vec!["change", "legacy"]);
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn ready() -> bool { false }\n"
    );
}

// Verifies REQ-change-039.
#[test]
fn batch_owner_corrections_append_transactionally_or_not_at_all() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() {}\n").unwrap();
    fs::write(root.join("src/b.rs"), "pub fn b() {}\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    let owned = |module: &str| {
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/a.rs\n  - src/b.rs\n  - src/lib.rs\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
        )
    };
    fs::write(root.join("specs/legacy/legacy.spec.md"), owned("legacy")).unwrap();
    fs::write(root.join("specs/current/current.spec.md"), owned("current")).unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "batch owner correction".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["legacy".into()],
            affected_paths: vec!["src/a.rs".into(), "src/b.rs".into(), "src/lib.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("ownership evidence only".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Owners can be batch-corrected".into()];
    record.answers.insert("public_contract".into(), "no".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    persist_legacy_test_record(root, &mut record);
    write_change_markdown(root, &record).unwrap();
    for artifact in &record.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Sources changed during release review".into(),
    )
    .unwrap()
    .change;
    let state_path = change_dir(root, &record.id).join("state.json");
    let before = fs::read(&state_path).unwrap();

    let failed = add_acceptance_owner_corrections(
        root,
        &record.id,
        vec![
            ("src/a.rs".into(), "current".into()),
            ("outside/x.rs".into(), "current".into()),
        ],
        "Release reviewer".into(),
        "Batch repair".into(),
    )
    .unwrap_err();
    assert!(
        failed.contains("batch entry 2 failed")
            || failed.contains("outside the original affected path scope"),
        "{failed}"
    );
    assert_eq!(fs::read(&state_path).unwrap(), before);

    record = add_acceptance_owner_corrections(
        root,
        &record.id,
        vec![
            ("src/a.rs".into(), "current".into()),
            ("src/b.rs".into(), "current".into()),
        ],
        "Release reviewer".into(),
        "Batch repair".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 2);
    assert_eq!(record.acceptance_owner_corrections[0].sequence, 1);
    assert_eq!(record.acceptance_owner_corrections[1].sequence, 2);

    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# Legacy\n\n## Purpose\n\nEmpty.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let missing = missing_acceptance_owner_paths(root, &record.id, "current").unwrap();
    assert_eq!(missing, vec!["src/lib.rs"]);
    record = add_acceptance_owner_corrections(
        root,
        &record.id,
        missing
            .into_iter()
            .map(|path| (path, "current".into()))
            .collect(),
        "Release reviewer".into(),
        "Discover remaining".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 3);
    assert!(
        missing_acceptance_owner_paths(root, &record.id, "current")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn owner_batch_validation_queries_canonical_module_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let paths = (0..20)
        .map(|index| format!("src/owner_{index}.rs"))
        .collect::<Vec<_>>();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    for path in &paths {
        fs::write(root.join(path), "pub fn owned() {}\n").unwrap();
    }
    let files = paths
        .iter()
        .map(|path| format!("  - {path}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
            root.join("specs/current/current.spec.md"),
            format!(
                "---\nmodule: current\nversion: 1\nstatus: stable\nfiles:\n{files}\n---\n\n# Current\n\n## Purpose\n\nOwner fixture.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
            ),
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    record.affected_paths = paths.clone();
    record.acceptance_owner_corrections = paths
        .iter()
        .enumerate()
        .map(|(index, path)| AcceptanceOwnerCorrection {
            schema_version: 1,
            sequence: index as u64 + 1,
            path: path.clone(),
            module: "current".into(),
            actor: "Reviewer".into(),
            reason: "Repair historical ownership".into(),
            timestamp: 1,
        })
        .collect();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "owner fixture"]);

    reset_test_canonical_module_query_count();
    reset_test_git_process_count();
    validate_acceptance_owner_corrections_current(root, &record).unwrap();
    let git_queries = test_git_process_count();

    assert_eq!(test_canonical_module_query_count(), 1);
    assert!(
        git_queries <= 40,
        "one module should require one bounded evidence capture, observed {git_queries} Git queries"
    );
}

// Verifies REQ-change-033.
#[test]
fn owner_correction_rejects_invalid_requests_without_mutation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let state_path = change_dir(root, &record.id).join("state.json");
    let before = fs::read(&state_path).unwrap();

    let wrong_state = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Reviewer".into(),
        "Missing owner".into(),
    )
    .unwrap_err();
    assert!(wrong_state.contains("cannot correct an acceptance input owner"));
    assert_eq!(fs::read(&state_path).unwrap(), before);

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(root, &record.id, "Reviewer".into(), "Source changed".into())
        .unwrap()
        .change;
    let reopened = fs::read(&state_path).unwrap();
    let error = add_acceptance_owner_correction(
        root,
        &record.id,
        "outside/file.rs".into(),
        "change".into(),
        "Reviewer".into(),
        "Missing owner".into(),
    )
    .unwrap_err();
    assert!(
        error.contains("outside the original affected path scope"),
        "{error}"
    );
    assert_eq!(fs::read(&state_path).unwrap(), reopened);
}

#[test]
fn broad_successor_without_explicit_obligations_cannot_suppress_stale_predecessor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut predecessor = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nOriginal governed behavior.\n",
    );
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/auth-extra.rs"),
        "// Existing product surface.\n",
    )
    .unwrap();
    let spec_path = root.join("specs/auth/auth.spec.md");
    let expanded = fs::read_to_string(&spec_path).unwrap().replace(
        "  - src/auth.rs\n",
        "  - src/auth.rs\n  - src/auth-extra.rs\n",
    );
    fs::write(&spec_path, expanded).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    let mut successor = create_change(
        root,
        CreateChangeRequest {
            description: "Expand the governed auth surface".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into(), "src/auth-extra.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    successor.acceptance_criteria = vec!["Both auth files remain governed".into()];
    successor
        .answers
        .insert("public_contract".into(), "yes".into());
    successor
        .answers
        .insert("architecture_risk".into(), "no".into());
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for artifact in &successor.selected_artifacts {
        fs::write(
            change_dir(root, &successor.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Govern the expanded surface.\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed successor evidence.\n")
            },
        )
        .unwrap();
    }
    fs::write(
        delta_path(root, &successor, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nBoth existing auth files remain governed.\n",
    )
    .unwrap();
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    successor = start_implementation(root, &successor.id).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    // A failing verification must not clear the stale-predecessor finding. The
    // failure has to sit inside the successor's own scope — `change check` is
    // scoped, so drift in a module it neither declares nor maps is ignored.
    // `auth_extra` maps `src/auth-extra.rs`, one of the successor's declared
    // paths, so the scoped spec↔code pass is what fails.
    let drifted_spec = root.join("specs/auth_extra/auth_extra.spec.md");
    fs::create_dir_all(drifted_spec.parent().unwrap()).unwrap();
    fs::write(
        &drifted_spec,
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: auth_extra")
            .replace("src/auth.rs", "src/auth-extra.rs")
            .replace("# Auth", "# Auth Extra"),
    )
    .unwrap();
    assert!(verify_change(root, &successor.id).is_err());
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    fs::remove_file(&drifted_spec).unwrap();
    verify_change(root, &successor.id).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));
    successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    assert_eq!(successor.state, ChangeState::Accepted);
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));
}

#[test]
fn reopen_rejects_current_evidence_and_requires_explicit_audit_fields() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    let error = reopen_change(root, &record.id, "Reviewer".into(), "Not stale".into()).unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let error = reopen_change(root, &record.id, " ".into(), "Review fix".into()).unwrap_err();
    assert!(error.contains("non-empty human actor"), "{error}");
    let error = reopen_change(root, &record.id, "Reviewer".into(), " ".into()).unwrap_err();
    assert!(error.contains("non-empty reason"), "{error}");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
}

// A legacy acceptance can sign the current content while its recorded transition
// contains different content. Restoring the signed content leaves reopen's old
// digest/anchor predicate satisfied, but archive cannot reconstruct the transition.
fn legacy_reopen_fixture(root: &Path, mismatch_at_transition: bool) -> ChangeRecord {
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implementation"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    let mut verification = load_verification(root, &record).unwrap();
    verification.acceptance_input_digest =
        Some(acceptance_input_digest(root, &record, &[]).unwrap());
    verification.acceptance_manifest = None;
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &verification,
    )
    .unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger
        .approvals
        .iter_mut()
        .rev()
        .find(|a| a.gate == "acceptance")
        .unwrap()
        .digest = closing_digest(&record, &verification);
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    let source = fs::read(root.join("src/lib.rs")).unwrap();
    if mismatch_at_transition {
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-m", "legacy acceptance transition"]);
    if mismatch_at_transition {
        fs::write(root.join("src/lib.rs"), source).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "restore signed delivery content"]);
    }
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    assert_eq!(
        verification.acceptance_input_digest.as_deref(),
        Some(
            acceptance_input_digest(root, &record, &[])
                .unwrap()
                .as_str()
        )
    );
    assert!(
        authenticate_accepted_evidence_with_anchor(root, &record, None)
            .unwrap()
            .1
    );
    record
}

#[test]
fn legacy_reopen_recovers_unreconstructible_current_acceptance() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = legacy_reopen_fixture(root, true);
    let prior = load_verification(root, &record).unwrap();
    let error = archive_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("cannot reproduce its signed raw-content aggregate"),
        "{error}"
    );
    let reopened = reopen_change(
        root,
        &record.id,
        "Reviewer".into(),
        "Refresh unreconstructible legacy acceptance (#751)".into(),
    )
    .unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.stale_acceptance_input_digest,
        reopened.audit.current_acceptance_input_digest
    );
    assert_eq!(
        serde_json::to_value(&reopened.audit).unwrap()["stale_evidence_cause"],
        "legacy_acceptance_unreconstructible"
    );
    assert_eq!(
        serde_json::to_value(&reopened.audit.prior_verification).unwrap(),
        serde_json::to_value(prior).unwrap()
    );
    assert!(reopened_change_preserves_sequence_history(
        root,
        &reopened.change
    ));
    verify_change(root, &record.id).unwrap();
    accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert!(
        load_verification(root, &record)
            .unwrap()
            .acceptance_manifest
            .is_some()
    );
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["add", "."]);
    git(&["commit", "-m", "record refreshed acceptance"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    archive_change(root, &record.id).unwrap();
}

#[test]
fn legacy_reopen_refuses_reconstructible_current_acceptance_without_mutation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = legacy_reopen_fixture(root, false);
    assert!(resolved_acceptance_manifest(root, &record).is_ok());
    let dir = change_dir(root, &record.id);
    let state = fs::read(dir.join("state.json")).unwrap();
    let ledger = fs::read(dir.join("approvals.json")).unwrap();
    let error =
        reopen_change(root, &record.id, "Reviewer".into(), "Still current".into()).unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
    assert_eq!(fs::read(dir.join("state.json")).unwrap(), state);
    assert_eq!(fs::read(dir.join("approvals.json")).unwrap(), ledger);
    archive_change(root, &record.id).unwrap();
}

#[test]
fn legacy_reopen_reconstruction_failure_does_not_bypass_authentication() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = legacy_reopen_fixture(root, true);
    let dir = change_dir(root, &record.id);
    let state = fs::read(dir.join("state.json")).unwrap();
    let approvals = fs::read(dir.join("approvals.json")).unwrap();
    for (actor, reason, expected) in [
        ("", "Refresh legacy evidence", "non-empty human actor"),
        ("Reviewer", "", "non-empty reason"),
    ] {
        let error = reopen_change(root, &record.id, actor.into(), reason.into()).unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert_eq!(fs::read(dir.join("state.json")).unwrap(), state);
        assert_eq!(fs::read(dir.join("approvals.json")).unwrap(), approvals);
    }
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger
        .approvals
        .iter_mut()
        .rev()
        .find(|a| a.gate == "acceptance")
        .unwrap()
        .digest = "0".repeat(64);
    write_json(&dir.join("approvals.json"), &ledger).unwrap();
    let tampered = fs::read(dir.join("approvals.json")).unwrap();
    let error = reopen_change(
        root,
        &record.id,
        "Reviewer".into(),
        "Refresh legacy evidence".into(),
    )
    .unwrap_err();
    assert!(
        error.contains("closing approval does not match verification evidence"),
        "{error}"
    );
    assert_eq!(fs::read(dir.join("state.json")).unwrap(), state);
    assert_eq!(fs::read(dir.join("approvals.json")).unwrap(), tampered);
}

/// A reopen that is correctly refused must not consume the dated archive package. The
/// un-archive is a move performed before the preconditions are known to hold, so every
/// failure after it has to put the package back; otherwise the refusal leaves an active
/// orphan whose state.json still says `archived` and no verb can recover it.
#[test]
fn refused_reopen_restores_the_archived_package() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept definition"]);
    let archived = archive_change(root, &record.id).unwrap();
    assert!(archived.is_dir());

    // Delivery inputs are still current, so this reopen is refused on purpose.
    let error = reopen_change(root, &record.id, "Reopener".into(), "probe".into()).unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
    assert!(error.contains("archive restored"), "{error}");
    assert!(
        archived.is_dir(),
        "the refused reopen consumed the archive package at {}",
        archived.display()
    );
    assert!(
        !change_dir(root, &record.id).exists(),
        "the refused reopen left an active orphan"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Archived
    );

    // The refusal stays the same diagnostic instead of degrading into an orphan collision.
    let retry = reopen_change(root, &record.id, "Reopener".into(), "retry".into()).unwrap_err();
    assert!(retry.contains("delivery inputs are current"), "{retry}");
    assert!(
        !retry.contains("an active change directory already exists"),
        "{retry}"
    );

    // Control: the un-archive is still reachable when the reopen can succeed, so the
    // restore above cannot be passing by never un-archiving at all.
    git(&["add", "."]);
    git(&["commit", "-m", "archive tip"]);
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "drift input"]);
    let reopened = reopen_change(root, &record.id, "Reopener".into(), "drifted".into()).unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert!(change_dir(root, &record.id).is_dir());
    assert!(!archived.exists());
}

#[test]
fn reaccept_rejects_definition_changes_after_canonical_application() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Review fixes changed scoped delivery inputs".into(),
    )
    .unwrap()
    .change;
    fs::write(
        change_dir(root, &record.id).join("testing.md"),
        "# Testing\n\nThe modified definition must not be silently ignored.\n",
    )
    .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains("modified definition of an already-applied change") })
    );
    verify_change(root, &record.id).unwrap();

    let error = accept_change(root, &record.id, Some("Closer".into()), None).unwrap_err();
    assert!(
        error.contains("perform further spec changes in a new change workspace"),
        "{error}"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Verifying
    );
}

// Verifies REQ-change-032.
#[test]
fn accepted_metadata_correction_preserves_original_evidence_and_adds_artifacts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let original_answers = record.answers.clone();
    let original_artifacts = record.selected_artifacts.clone();
    let original_approvals = serde_json::to_value(load_approvals(root, &record).unwrap()).unwrap();
    let prior_verification = load_verification(root, &record).unwrap();

    let result = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap();

    assert_eq!(result.change.state, ChangeState::Verifying);
    assert!(result.change.canonical_applied);
    assert_eq!(result.change.correction_count, 1);
    assert_eq!(result.change.answers, original_answers);
    assert_eq!(result.change.selected_artifacts, original_artifacts);
    assert_eq!(result.correction.original_value, "no");
    assert_eq!(result.correction.prior_effective_value, "no");
    assert_eq!(result.correction.corrected_value, "yes");
    assert_eq!(
        result.correction.prior_verification.contract_digest,
        prior_verification.contract_digest
    );
    assert_eq!(
        result.correction.added_artifacts,
        vec![
            ArtifactKind::Research,
            ArtifactKind::Design,
            ArtifactKind::Plan,
        ]
    );
    assert_eq!(
        result
            .effective_definition
            .answers
            .get("architecture_risk")
            .map(String::as_str),
        Some("yes")
    );
    for artifact in ["research.md", "design.md", "plan.md"] {
        assert!(result.summary.next_action.contains(artifact));
    }
    assert!(!result.summary.approval_valid);
    assert_eq!(
        serde_json::to_value(load_approvals(root, &result.change).unwrap()).unwrap(),
        original_approvals
    );
    for artifact in &result.correction.added_artifacts {
        let content =
            fs::read_to_string(change_dir(root, &record.id).join(artifact.file_name())).unwrap();
        assert!(content.contains("<!-- TODO"));
    }
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Release reviewer".into(),
        "A second correction cannot bypass reacceptance".into(),
    )
    .unwrap_err();
    assert!(error.contains("expected accepted"), "{error}");
}

#[test]
fn metadata_correction_rejects_noops_unsupported_fields_and_missing_audit_inputs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));

    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "no".into(),
        "Reviewer".into(),
        "No change".into(),
    )
    .unwrap_err();
    assert!(error.contains("already `no`"), "{error}");
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        " ".into(),
        "Missing actor".into(),
    )
    .unwrap_err();
    assert!(error.contains("non-empty human actor"), "{error}");
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        " ".into(),
    )
    .unwrap_err();
    assert!(error.contains("non-empty reason"), "{error}");
    assert!(CorrectionField::parse("acceptance_criteria").is_err());
    assert!(!change_dir(root, &record.id).join(CORRECTIONS_FILE).exists());
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
}

#[test]
fn correction_values_preserve_supported_boolean_aliases() {
    for value in ["yes", "y", "true", "1", " YES "] {
        assert_eq!(canonical_correction_value(value).unwrap(), "yes");
    }
    for value in ["no", "n", "false", "0", " NO "] {
        assert_eq!(canonical_correction_value(value).unwrap(), "no");
    }
    assert!(canonical_correction_value("maybe").is_err());
}

#[test]
fn corrected_acceptance_requires_fresh_gates_and_never_replays_canonical_deltas() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nCorrected metadata never replays this canonical section.\n";
    let mut record = completed_section_only_record(root, delta);
    record.answers.insert("public_contract".into(), "no".into());
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = accept_completed_record(root, record);
    let canonical_path = root.join("specs/auth/auth.spec.md");
    let canonical_after_first_accept = fs::read_to_string(&canonical_path).unwrap();

    let first = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted semantic delta changed the public contract".into(),
    )
    .unwrap();
    assert!(first.correction.added_artifacts.is_empty());
    assert_eq!(
        first.summary.next_action,
        format!("run `specsync change approve {} --actor <name>`", record.id)
    );
    assert!(ensure_definition_approval_valid(root, &first.change).is_err());
    record =
        approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
    assert_eq!(
        fs::read_to_string(&canonical_path).unwrap(),
        canonical_after_first_accept
    );
    assert_eq!(correction_history(root, &record).unwrap().len(), 1);

    let second = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "no".into(),
        "Release reviewer".into(),
        "A later audit restored the original classification".into(),
    )
    .unwrap();
    assert_eq!(second.correction.sequence, 2);
    record =
        approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();

    assert_eq!(
        record.answers.get("public_contract").map(String::as_str),
        Some("no")
    );
    assert_eq!(record.correction_count, 2);
    assert_eq!(correction_history(root, &record).unwrap().len(), 2);
    assert_eq!(
        effective_change_definition(root, &record)
            .unwrap()
            .answers
            .get("public_contract")
            .map(String::as_str),
        Some("no")
    );
    assert_eq!(
        fs::read_to_string(canonical_path).unwrap(),
        canonical_after_first_accept
    );
}

// Verifies REQ-change-032.
#[test]
fn trusted_history_rejects_correction_rollback_and_divergent_same_count() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nAccepted passkey changes retain lifecycle evidence.\n",
    );
    record.answers.insert("public_contract".into(), "no".into());
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    let first = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted implementation has architectural risk".into(),
    )
    .unwrap();
    for artifact in &first.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept first correction"]);
    let first_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let second = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "no".into(),
        "Reviewer".into(),
        "A follow-up audit removed the architectural risk".into(),
    )
    .unwrap();
    assert_eq!(second.correction.sequence, 2);
    assert!(effective_change_definition(root, &second.change).is_ok());
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept second correction"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let second_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let workspace = format!("{CHANGES_PATH}/{}", record.id);

    git(&[
        "restore",
        "--source",
        first_commit.as_str(),
        "--",
        &workspace,
    ]);
    let rolled_back = load_change(root, &record.id).unwrap();
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
    git(&[
        "restore",
        "--source",
        second_commit.as_str(),
        "--",
        &workspace,
    ]);

    git(&["switch", "-c", "divergent", first_commit.as_str()]);
    let stale = load_change(root, &record.id).unwrap();
    let error = effective_change_definition(root, &stale).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
    git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    let divergent = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Reviewer".into(),
        "The public contract classification was corrected independently".into(),
    )
    .unwrap();
    git(&[
        "update-ref",
        "refs/remotes/origin/main",
        second_commit.as_str(),
    ]);
    let error = effective_change_definition(root, &divergent.change).unwrap_err();
    assert!(
        error.contains("correction history divergence detected"),
        "expected correction history divergence"
    );
}

// Verifies REQ-change-032.
#[test]
fn full_history_finds_a_corrected_anchor_hidden_by_a_treesame_merge_result() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);
    let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    git(&["switch", "-c", "corrected-side-branch"]);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(&[
        "restore",
        "--source",
        original.as_str(),
        "--",
        workspace.as_str(),
    ]);
    git(&["add", "."]);
    git(&["commit", "-m", "roll back corrected workspace"]);
    git(&["switch", "main"]);
    git(&[
        "merge",
        "--no-ff",
        "corrected-side-branch",
        "-m",
        "merge side-branch history",
    ]);

    let rolled_back = load_change(root, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn trusted_history_ignores_a_dangling_remote_default_symbolic_ref() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    git(&["add", "."]);
    git(&["commit", "-m", "accept definition"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/missing",
    ]);

    assert!(effective_change_definition(root, &record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn historical_git_paths_are_nul_safe_in_a_non_ascii_project_directory() {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    let fixture = if cfg!(windows) {
        "fixtures/naïve quoted"
    } else {
        "fixtures/naïve \"quoted\""
    };
    let root = repository.join(fixture);
    fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repository)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(&root);
    let mut record = completed_section_only_record(
        &root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nHistorical paths remain byte-delimited.\n",
    );
    record.answers.insert("public_contract".into(), "no".into());
    save_change(&root, &record).unwrap();
    write_change_markdown(&root, &record).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(&root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    record = correct_interview_metadata(
        &root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Reviewer".into(),
        "The accepted semantic delta changes a public contract".into(),
    )
    .unwrap()
    .change;
    record = approve_definition(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(&root, &record.id).unwrap();
    record = accept_change(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);

    let commit = git_output(&root, &["rev-parse", "HEAD"]).unwrap();
    let directory =
        git_repo_relative_path(&root, &format!("{CHANGES_PATH}/{}", record.id)).unwrap();
    assert_eq!(
        historical_change_directories(&root, &commit, &record.id).unwrap(),
        vec![directory.clone()]
    );
    assert!(
        closing_authenticated_correction_anchor(&root, &commit, &directory, &record.id)
            .unwrap()
            .is_some()
    );
    assert!(
        git_entry_at_commit(&root, &commit, &format!("{directory}/state.json"))
            .unwrap()
            .is_some()
    );
    assert!(effective_change_definition(&root, &record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn archived_change_uses_prior_active_correction_anchor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let archive = archive_change(root, &record.id).unwrap();
    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    assert!(effective_change_definition(root, &archived).is_ok());

    let mut ledger = load_correction_ledger(root, &archived).unwrap();
    ledger.corrections.clear();
    write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
    let mut rolled_back = archived;
    rolled_back.correction_count = 0;
    write_json(&archive.join("state.json"), &rolled_back).unwrap();
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn archived_only_corrected_snapshot_remains_a_trusted_anchor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    let archive = archive_change(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record only archived corrected snapshot"]);
    assert!(
        git_output(
            root,
            &[
                "log",
                "--format=%H",
                "--",
                &format!("{CHANGES_PATH}/{}/corrections.json", record.id),
            ],
        )
        .is_none()
    );

    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    let mut ledger = load_correction_ledger(root, &archived).unwrap();
    ledger.corrections.clear();
    write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
    let mut rolled_back = archived;
    rolled_back.correction_count = 0;
    write_json(&archive.join("state.json"), &rolled_back).unwrap();

    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn shallow_history_with_corrections_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    let head = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    fs::write(root.join(".git/shallow"), format!("{head}\n")).unwrap();
    let error = effective_change_definition(root, &correction.change).unwrap_err();
    assert!(
        error.contains("shallow Git checkout"),
        "expected shallow Git checkout rejection"
    );
}

// Verifies REQ-change-032.
#[test]
fn shallow_rollback_tip_cannot_hide_a_corrected_acceptance() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let clone = temp.path().join("clone");
    fs::create_dir_all(&source).unwrap();
    let git = |root: &Path, args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {} failed",
            args.join(" ")
        );
    };
    git(&source, &["init", "-b", "main"]);
    git(&source, &["config", "user.email", "test@example.com"]);
    git(&source, &["config", "user.name", "Test"]);
    fs::write(source.join("README.md"), "# Fixture\n").unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "base"]);
    write_lifecycle_test_policy(&source);
    let mut record = completed_no_spec_record(&source);
    record = accept_completed_record(&source, record);
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "accept original definition"]);
    let original = git_output(&source, &["rev-parse", "HEAD"]).unwrap();

    let correction = correct_interview_metadata(
        &source,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(&source, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(&source, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(&source, &record.id).unwrap();
    record = accept_change(&source, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "accept corrected definition"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(
        &source,
        &["restore", "--source", original.as_str(), "--", &workspace],
    );
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "rollback lifecycle workspace"]);
    let source_url = format!("file://{}", source.display());
    git(
        temp.path(),
        &[
            "clone",
            "--depth",
            "1",
            source_url.as_str(),
            clone.to_str().unwrap(),
        ],
    );
    let rolled_back = load_change(&clone, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    let error = effective_change_definition(&clone, &rolled_back).unwrap_err();
    assert!(error.contains("incomplete shallow Git checkout"), "{error}");

    let new_record = create_change(
        &clone,
        CreateChangeRequest {
            description: "Add a local shallow-checkout note".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Local documentation only".into()),
        },
    )
    .unwrap();
    assert!(effective_change_definition(&clone, &new_record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn accepted_snapshot_with_a_stale_contract_is_not_an_anchor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);
    let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                prose_or_plan(
                    artifact,
                    "# Complete\n\nThe corrected classification was reviewed.\n",
                )
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    fs::write(
        change_dir(root, &record.id).join("testing.md"),
        "# Complete\n\nChanged after the accepted contract was approved.\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record stale accepted contract snapshot"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(&["restore", "--source", original.as_str(), "--", &workspace]);
    let rolled_back = load_change(root, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    assert!(effective_change_definition(root, &rolled_back).is_ok());
}

#[test]
fn correction_ledgers_fail_closed_and_hash_portably() {
    let first = TempDir::new().unwrap();
    let root = first.path();
    write_lifecycle_test_policy(root);
    let accepted = accept_completed_record(root, completed_no_spec_record(root));
    let result = correct_interview_metadata(
        root,
        &accepted.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "Architecture classification was stale".into(),
    )
    .unwrap();
    let record = result.change;
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let valid_ledger = fs::read_to_string(&ledger_path).unwrap();

    fs::remove_file(&ledger_path).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("does not match state.json correction_count"),
        "expected correction_count mismatch"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut tampered: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    tampered["corrections"][0]["sequence"] = serde_json::json!(2);
    write_json(&ledger_path, &tampered).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("sequence is not contiguous"),
        "expected non-contiguous correction sequence"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut unsupported: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    unsupported["corrections"][0]["field"] = serde_json::json!("acceptance_criteria");
    write_json(&ledger_path, &unsupported).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("invalid correction ledger"),
        "expected invalid correction ledger"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut tampered_definition: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    tampered_definition["corrections"][0]["superseded_definition_approval"]["digest"] =
        serde_json::json!("forged-definition-digest");
    write_json(&ledger_path, &tampered_definition).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("invalid prior gate evidence"),
        "expected invalid prior gate evidence"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let second = TempDir::new().unwrap();
    let second_root = second.path();
    let second_dir = change_dir(second_root, &record.id);
    fs::create_dir_all(second_dir.join("deltas")).unwrap();
    save_change(second_root, &record).unwrap();
    fs::write(second_dir.join(CORRECTIONS_FILE), valid_ledger).unwrap();
    let effective = effective_change_definition(root, &record).unwrap();
    for artifact in &effective.selected_artifacts {
        let content = fs::read(change_dir(root, &record.id).join(artifact.file_name())).unwrap();
        fs::write(second_dir.join(artifact.file_name()), content).unwrap();
    }
    assert_eq!(
        definition_digest(root, &record).unwrap(),
        definition_digest(second_root, &record).unwrap()
    );
}

// Verifies REQ-change-056.
#[test]
fn text_correction_ledger_health_hides_invalid_ledger_detail() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = completed_no_spec_record(root);
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);

    assert!(effective_change_definition(root, &record).is_ok());

    fs::write(&ledger_path, "{ malformed correction ledger\n").unwrap();

    assert!(effective_change_definition(root, &record).is_err());
}

// Verifies REQ-change-057.
#[test]
fn mutation_rechecks_correction_ledger_after_lock_acquisition() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = completed_no_spec_record(root);
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let state_path = change_dir(root, &record.id).join("state.json");
    let change_path = change_dir(root, &record.id).join("change.md");

    let project_lock = acquire_project_lock(root).unwrap();
    let mutation_root = root.to_path_buf();
    let mutation_id = record.id.clone();
    let (started_sender, started_receiver) = std::sync::mpsc::channel();
    let mutation = std::thread::spawn(move || {
        started_sender.send(()).unwrap();
        answer_question(
            &mutation_root,
            &mutation_id,
            "acceptance_criteria",
            "This mutation must not persist",
        )
    });

    started_receiver.recv().unwrap();
    fs::write(&ledger_path, "{ malformed correction ledger\n").unwrap();
    let state_before = fs::read(&state_path).unwrap();
    let change_before = fs::read(&change_path).unwrap();
    drop(project_lock);

    let error = mutation.join().unwrap().unwrap_err();
    assert_eq!(error, INVALID_CORRECTION_LEDGER_TEXT);
    assert_eq!(fs::read(&state_path).unwrap(), state_before);
    assert_eq!(fs::read(&change_path).unwrap(), change_before);
}

#[test]
fn acceptance_rechecks_late_dependency_state() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let dependency = create_change(
        root,
        CreateChangeRequest {
            description: "unfinished prerequisite".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Ordering fixture".into()),
        },
    )
    .unwrap();
    record = load_change(root, &record.id).unwrap();
    record.dependencies.push(dependency.id.clone());
    save_change(root, &record).unwrap();
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        definition_digest(root, &record).unwrap(),
        Some("Approved late ordering change".into()),
    )
    .unwrap();
    let mut evidence = load_verification(root, &record).unwrap();
    evidence.contract_digest = definition_digest(root, &record).unwrap();
    evidence.execution_digest = Some(execution_digest(root, &record).unwrap());
    evidence.workspace_digest = project_input_digest(root).unwrap();
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &evidence,
    )
    .unwrap();
    let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(error.contains("must be accepted"), "{error}");
}

#[test]
fn failed_evidence_keeps_local_check_red() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    let mut evidence = verify_change(root, &record.id).unwrap();
    evidence.passed = false;
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &evidence,
    )
    .unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("latest verification evidence failed"))
    );
}
