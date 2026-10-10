use super::super::repo_status::classify_paths;
use super::super::ship_status::{
    merge_before_finalize_warning, multi_active_ordering_warnings, ship_status_report,
};
use super::fixtures::{draft_fixture, git_in, git_project_fixture, reviewed_change_fixture};
use crate::change;
use crate::change::ChangeState;
use std::fs;
use tempfile::TempDir;

/// Honest label: CONTROL for the two tests below, and it passes on the unfixed binary
/// too — that is exactly why it is here.
///
/// #743 asks readiness to consult review currency, and the cheap way to satisfy any test
/// that demands `ready_to_finalize: false` is to make the conjunction stricter until
/// nothing is ever ready. This pins the other side: a change whose review was recorded
/// against this very tree, with nothing touched since, must still reach
/// `ready_to_finalize: true`.
///
/// What breaks this test: adding any further term to the `ready_to_finalize` conjunction
/// that a healthy, just-reviewed change cannot satisfy — most obviously a bare
/// `&& scoped_review_current` implemented as "the descendant walk must have proven
/// something", since on a squash-merging repository that walk is unavailable rather than
/// satisfied and the whole class of changes goes permanently red. That is the failure
/// #689 removed from the verification half, and trading a false green for a permanent
/// false red is not a fix.
#[test]
fn ship_status_is_ready_when_the_scoped_review_is_current() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);

    let id = reviewed_change_fixture(root);
    let record = change::load_change(root, &id).expect("reload");
    let report = ship_status_report(root, &record).expect("ship status");

    assert_eq!(
        report["ready_to_finalize"], true,
        "a healthy just-reviewed change must still be ready to finalize: {report}"
    );
    let blockers = report["blockers"].as_array().expect("blockers");
    assert!(
        blockers.is_empty(),
        "a current review must raise no blocker: {blockers:?}"
    );
}

/// Honest label: TRUE DISCRIMINATOR for #743.
///
/// `ready_to_finalize` asked `review_path.is_file()` — existence — while `finalize`
/// additionally required `scoped_review_is_current`. So ship-status recommended
/// `specsync change ship` and `finalize` refused it one command later, with only a
/// read-only command in between.
///
/// The staleness here is genuine and by CONTENT: the implementation changes after the
/// review is recorded, and verification is then re-run so that the verification half of
/// the conjunction is healthy again. That isolation matters — every content digest the
/// review checks is also checked by verification, so without the re-check the
/// verification blocker alone would make readiness false and the test would prove
/// nothing.
///
/// Discriminates: on a binary built from unfixed `main` this fails with
/// `ready_to_finalize` = true while `finalize_change` returns
/// `Err("independent scoped review is stale; ...")`.
///
/// The assertion is AGREEMENT between the two commands, not `ready_to_finalize == false`.
/// #694 has three live options for the unavailable case and all three end with these two
/// agreeing; gating on a particular value would go red the day #694 lands the other way.
#[test]
fn ship_status_and_finalize_agree_when_the_review_is_stale_by_content() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);

    let id = reviewed_change_fixture(root);

    // The implementation genuinely moves after the review, and verification catches up.
    fs::write(root.join("README.md"), "# fixture\n\nsecond thoughts\n").unwrap();
    git_in(root, &["add", "."]);
    git_in(
        root,
        &["commit", "-m", "change the implementation after the review"],
    );
    change::check_change(root, Some(&id)).expect("re-check");
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "record fresh verification"]);

    let record = change::load_change(root, &id).expect("reload");
    let report = ship_status_report(root, &record).expect("ship status");
    assert_eq!(
        report["verification_present"], true,
        "premise: verification itself is healthy, so only the review half is in question: {report}"
    );

    let ready = report["ready_to_finalize"] == serde_json::Value::Bool(true);
    let finalize = change::finalize_change(root, &id);

    assert_eq!(
        ready,
        finalize.is_ok(),
        "ship-status and finalize disagree about the same change in the same second: ready_to_finalize={ready}, finalize={finalize:?}, report={report}"
    );

    // Agreement is worth nothing if the two commands agree by accident on unrelated
    // grounds, so the refusal `finalize` produced has to be the scoped-review one this
    // test is about. Unconditional here, unlike the squash case: a content change after
    // the review is a decided negative under every option #694 has on the table.
    assert!(
        finalize
            .as_ref()
            .err()
            .is_some_and(|error| error.contains("independent scoped review is stale")),
        "the agreement must be about the review, not some other refusal: {finalize:?}"
    );
    // A decided negative has to name what moved, or the reader is told "not ready" and
    // left to guess (#689's content framing applied to the review half).
    assert_eq!(
        report["review_currency"].as_str(),
        Some("stale"),
        "a content change after the review is a DECIDED negative, not an unavailable one: {report}"
    );
    let blockers: Vec<&str> = report["blockers"]
        .as_array()
        .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
        .unwrap_or_default();
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.contains("scoped review is stale")),
        "readiness must name the stale review it is refusing on: {blockers:?}"
    );
}

/// Honest label: DISCRIMINATOR for #689, relabelled by #743.
///
/// #689: a squash-merge rewrites the recorded verification commit, so
/// `merge-base --is-ancestor` can never hold again — and this repository, like most,
/// permits only squash merges. Before that fix `ready_to_finalize` required the
/// ancestry, so a squash-merged change was permanently unfinalizable while its evidence
/// was perfectly good. That half is asserted unchanged below: no verification blocker,
/// and `verification_current` still true.
///
/// What #743 changed is the OTHER half. This test used to assert `ready_to_finalize:
/// true` here, which was the defect written down as an expectation: `finalize` refuses on
/// this exact tree with "independent scoped review is stale", because the squash also
/// destroyed `review.implementation_commit` and the descendant walk cannot run. The
/// guarantee is unavailable, not violated, and #694 states the standard — "an unavailable
/// guarantee reported as a satisfied one is worse than the current failure".
///
/// So the assertion is AGREEMENT, exactly as in the stale case, and it survives whichever
/// way #694 is resolved: if the walk gains a content fallback, `finalize` starts
/// succeeding and readiness follows it back to `true` through the same predicate.
#[test]
fn ship_status_and_finalize_agree_after_a_squash_that_preserves_content() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);
    git_in(root, &["switch", "-c", "feature"]);

    let id = reviewed_change_fixture(root);

    // Squash onto main: content identical, recorded commits unreachable. Exactly what
    // GitHub does, and the only strategy many repositories permit.
    git_in(root, &["switch", "main"]);
    git_in(root, &["merge", "--squash", "feature"]);
    git_in(root, &["commit", "-m", "squash feature"]);

    let record = change::load_change(root, &id).expect("reload");
    let report = ship_status_report(root, &record).expect("ship status");

    assert_eq!(
        report["verification_ancestor_of_head"], false,
        "the premise: the squash destroyed the recorded commit"
    );

    assert_eq!(report["stages"][0]["id"], "product_tip");
    assert_eq!(
        report["stages"][0]["status"], "done",
        "current content must complete the product stage despite rewritten ancestry: {report}"
    );
    assert_eq!(
        report["stages"][0]["action"],
        "product verification evidence is current for this content"
    );

    // #689, untouched: the verification half is a content question and the content is
    // intact, so nothing about verification may block.
    let blockers: Vec<&str> = report["blockers"]
        .as_array()
        .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !blockers
            .iter()
            .any(|blocker| blocker.contains("verification")),
        "a preserved-content squash must raise no verification blocker: {blockers:?}"
    );

    let ready = report["ready_to_finalize"] == serde_json::Value::Bool(true);
    let finalize = change::finalize_change(root, &id);
    assert_eq!(
        ready,
        finalize.is_ok(),
        "ship-status and finalize disagree about the same change in the same second: ready_to_finalize={ready}, finalize={finalize:?}, report={report}"
    );

    // Guards against agreeing by accident, and phrased so that #694 resolving the other
    // way clears it rather than reddening it: IF finalize refused, the refusal has to be
    // the scoped-review one.
    if let Err(error) = &finalize {
        assert!(
            error.contains("independent scoped review is stale"),
            "the agreement must be about the review, not some other refusal: {error}"
        );
    }

    // CHARACTERIZATION of #694's open state, not part of the gate. Today the walk is
    // unobtainable after a squash and readiness says so instead of rounding it to either
    // of the other two answers. If #694 lands a content fallback, this line becomes
    // `current` and should be updated — the agreement assertion above should not.
    assert_eq!(
        report["review_currency"].as_str(),
        Some("unavailable"),
        "a squash makes the descendant walk unobtainable, not violated: {report}"
    );
}

#[test]
fn ship_status_product_stage_rejects_stale_content_with_ancestor_evidence() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);
    let id = reviewed_change_fixture(root);
    fs::write(
        root.join("README.md"),
        "# fixture\n\nchanged after verification\n",
    )
    .unwrap();
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "change verified content"]);

    let record = change::load_change(root, &id).expect("reload");
    let report = ship_status_report(root, &record).expect("ship status");
    assert_eq!(report["verification_ancestor_of_head"], true);
    assert!(!change::recorded_verification_is_current(root, &record));
    assert_eq!(report["ready_to_finalize"], false);
    assert_eq!(report["stages"][0]["status"], "current", "{report}");
}

#[test]
fn ship_status_product_stage_rejects_missing_verification() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);
    let id = reviewed_change_fixture(root);
    fs::remove_file(
        root.join(".specsync/changes")
            .join(&id)
            .join("verification.json"),
    )
    .unwrap();
    let record = change::load_change(root, &id).expect("reload");
    let report = ship_status_report(root, &record).expect("ship status");
    assert_eq!(report["ready_to_finalize"], false);
    assert_eq!(report["stages"][0]["status"], "current", "{report}");
}

/// The ship lane may narrow the next action; it may never contradict the
/// lifecycle state (#534). At draft the lane wanted `change check --commit`
/// while the change was still in its interview, so obeying the printed line
/// failed against the same binary that printed it.
#[test]
fn draft_ship_next_defers_to_the_lifecycle_next_action() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let record = draft_fixture(root);

    let report = ship_status_report(root, &record).expect("ship status");
    let ship_next = report["ship_next"].as_str().expect("ship_next");
    let lifecycle_next = report["lifecycle_next"].as_str().expect("lifecycle_next");

    assert_eq!(ship_next, lifecycle_next, "{report}");
    assert!(
        !ship_next.contains("--commit"),
        "a draft must not be told to commit verification: {ship_next}"
    );
}

/// `ship_next` must name a runnable action, never restate a blocker. Blockers
/// already render on their own `Blocker:` lines, so repeating one here cost the
/// reader their next step and bought nothing (#534).
///
/// Honest label: this is an INVARIANT, not a discriminator. It passes on the
/// unfixed binary too, because a draft carries no blockers and the defect lives at
/// `Approved` — which this fixture cannot reach without driving a full interview and
/// approval. Drill 053's approved-state gate is what actually judges that half; this
/// guards against a future regression reintroducing the restatement anywhere.
#[test]
fn ship_next_is_an_action_never_a_blocker_restatement() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let record = draft_fixture(root);

    let report = ship_status_report(root, &record).expect("ship status");
    let ship_next = report["ship_next"].as_str().expect("ship_next");
    let blockers: Vec<&str> = report["blockers"]
        .as_array()
        .map(|values| values.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    for blocker in blockers {
        assert_ne!(
            ship_next, blocker,
            "Next: restated a blocker instead of naming an action: {report}"
        );
    }
}

/// Evidence must be resolved from wherever the change lives. Both reads used a
/// hard-coded `.specsync/changes/<id>/` — a parallel implementation of
/// `change_dir` that an archived change has moved out of — so a finalized change
/// reported `Verification: none` / `Review: missing` for artifacts sitting in its
/// own archive package (#534).
#[test]
fn archived_evidence_is_resolved_from_the_archive_package() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let record = draft_fixture(root);

    // Move the workspace to the archive exactly as finalize does, leaving the
    // active path empty, then teach the record it is archived.
    let active = root.join(".specsync/changes").join(&record.id);
    let archived = root
        .join(".specsync/archive/changes")
        .join(format!("2026-08-19-{}", record.id));
    fs::create_dir_all(archived.parent().expect("archive parent")).expect("archive root");
    fs::rename(&active, &archived).expect("archive the workspace");
    fs::write(
        archived.join("review.json"),
        "{\"reviewer\":\"peer\",\"verdict\":\"pass\"}\n",
    )
    .expect("write review evidence");
    fs::write(
        archived.join("verification.json"),
        format!("{{\"commit\":\"{}\",\"passed\":true}}\n", "a".repeat(40)),
    )
    .expect("write verification evidence");
    let mut archived_record = record.clone();
    archived_record.state = ChangeState::Archived;
    fs::write(
        archived.join("state.json"),
        serde_json::to_string(&archived_record).expect("serialize archived state"),
    )
    .expect("write archived state");

    let report = ship_status_report(root, &archived_record).expect("ship status");

    assert_eq!(
        report["verification_commit"].as_str(),
        Some("a".repeat(40).as_str()),
        "archived verification was not resolved: {report}"
    );
    assert_eq!(
        report["review_present"].as_bool(),
        Some(true),
        "archived review was not resolved: {report}"
    );
}

/// The read must be lenient. A strict `?` on the archived artifact turns
/// `ship-status` and `ship` from rc=0 into rc=1 on a repository whose evidence is
/// already damaged — the fix for an inspection command must not be the thing that
/// breaks inspection (#534).
#[test]
fn a_corrupt_archived_verification_degrades_instead_of_failing() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let record = draft_fixture(root);

    let active = root.join(".specsync/changes").join(&record.id);
    let archived = root
        .join(".specsync/archive/changes")
        .join(format!("2026-08-19-{}", record.id));
    fs::create_dir_all(archived.parent().expect("archive parent")).expect("archive root");
    fs::rename(&active, &archived).expect("archive the workspace");
    fs::write(archived.join("verification.json"), "not json at all\n")
        .expect("write corrupt evidence");
    let mut archived_record = record.clone();
    archived_record.state = ChangeState::Archived;
    fs::write(
        archived.join("state.json"),
        serde_json::to_string(&archived_record).expect("serialize archived state"),
    )
    .expect("write archived state");

    let report = ship_status_report(root, &archived_record)
        .expect("a corrupt archived artifact must not fail the command");

    assert!(
        report["verification_commit"].is_null(),
        "corrupt evidence must read as absent, not as a commit: {report}"
    );
}
#[test]
fn classify_paths_distinguishes_product_review_and_archive_tips() {
    assert_eq!(classify_paths(&[]), "other");
    assert_eq!(
        classify_paths(&["src/commands/change.rs".into()]),
        "product"
    );
    assert_eq!(
        classify_paths(&[
            ".specsync/changes/CHG-0001-demo/review.json".into(),
            ".specsync/changes/CHG-0001-demo/review-attempts.json".into(),
        ]),
        "review_only"
    );
    assert_eq!(
        classify_paths(&[
            ".specsync/archive/changes/2026-08-07-CHG-0001-demo/state.json".into(),
            ".specsync/archive/changes/2026-08-07-CHG-0001-demo/finalization.json".into(),
        ]),
        "archive_only"
    );
    assert_eq!(
        classify_paths(&[
            "src/cli.rs".into(),
            ".specsync/changes/CHG-0001-demo/review.json".into(),
        ]),
        "product"
    );
}

/// #687: the warning must name the cost that lands on OTHER changes, not only this one.
/// Pins the disclosure so a future refactor cannot quietly shrink it back to "strands the
/// change", which under-prices the decision it exists to inform.
#[test]
fn the_merge_warning_names_the_cost_to_earlier_accepted_changes() {
    for still_active in [false, true] {
        let warning = merge_before_finalize_warning(still_active);
        assert!(
            warning.contains("every earlier accepted change sharing a delivery input"),
            "must name whose work is blocked: {warning}"
        );
        assert!(
            warning.contains("from archiving"),
            "must name what is blocked: {warning}"
        );
        assert!(
            warning.contains("finalized or those are reopened"),
            "must name both exits: {warning}"
        );
    }
    assert!(merge_before_finalize_warning(false).contains("before finalize"));
    assert!(merge_before_finalize_warning(true).contains("still active"));
}

#[test]
fn multi_active_ordering_warnings_encode_four_rules() {
    assert!(multi_active_ordering_warnings(&[]).is_empty());
    let warnings = multi_active_ordering_warnings(&["CHG-0002-sibling".into()]);
    assert_eq!(warnings.len(), 3);
    let joined = warnings.join(" ");
    assert!(joined.contains("finalize one at a time"), "{joined}");
    assert!(joined.contains("do not batch reviews"), "{joined}");
    assert!(joined.contains("still active"), "{joined}");
    assert!(joined.contains("CHG-0002-sibling"), "{joined}");
}
