// Issue #633, audit D7: the relayed-decision record's own rules, below the CLI. The binary-driven
// tests in `tests/integration/relayed_decision.rs` cover the flags, status and finalize; these
// cover the shape Leif approved for both SpecSync and Chough, the review digest, and the
// library guards a JSON or agent caller meets without going through the flags.

const RELAY_DELTA: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials.\n";

fn relay(decided_by: &str) -> RelayedDecision {
    RelayedDecision {
        decided_by: decided_by.into(),
        words: "Approve it".into(),
        where_: "orc tab, relayed by t18".into(),
        recorded_by: "agent".into(),
    }
}

fn relay_json() -> serde_json::Value {
    serde_json::json!({
        "decided_by": "Ada",
        "words": "Approve it",
        "where": "orc tab, relayed by t18",
        "recorded_by": "agent",
    })
}

fn review_record(relayed_by: Option<RelayedDecision>) -> ScopedReviewRecord {
    ScopedReviewRecord {
        schema_version: 2,
        change_id: "add-auth".into(),
        reviewer: "Ada".into(),
        provenance: ScopedReviewProvenanceV1 {
            schema_version: 1,
            provider: ScopedReviewProvenanceProvider::GithubActionsCheck,
            required_check: SCOPED_REVIEW_REQUIRED_CHECK.into(),
        },
        verdict: ScopedReviewVerdict::Pass,
        implementation_commit: "a".repeat(40),
        contract_digest: "b".repeat(64),
        execution_digest: Some("d".repeat(64)),
        workspace_digest: "c".repeat(64),
        timestamp: 1_793_900_000,
        relayed_by,
    }
}

/// DISCRIMINATOR for the shared shape: exactly four keys, `where` spelled as in Chough, and
/// every way a hand-edited record breaks the shape is refused with a message naming `relayed_by`.
#[test]
fn a_relayed_decision_reads_only_the_shared_four_key_shape() {
    let read: RelayedDecision = serde_json::from_value(relay_json()).unwrap();
    assert_eq!(read, relay("Ada"));
    assert_eq!(serde_json::to_value(&read).unwrap(), relay_json());

    let mut cases: Vec<(serde_json::Value, &str)> = vec![
        (serde_json::Value::Null, "must be an object"),
        (serde_json::json!("Ada said yes"), "must be an object"),
    ];
    let mut unknown = relay_json();
    unknown["signed"] = serde_json::json!(true);
    cases.push((unknown, "unknown key `signed`"));
    for key in ["decided_by", "words", "where", "recorded_by"] {
        let mut missing = relay_json();
        missing.as_object_mut().unwrap().remove(key);
        cases.push((missing, "is missing"));
        let mut blank = relay_json();
        blank[key] = serde_json::json!(" \n ");
        cases.push((blank, "must not be blank"));
        let mut number = relay_json();
        number[key] = serde_json::json!(7);
        cases.push((number, "must be a string"));
        let mut long = relay_json();
        long[key] = serde_json::json!("x".repeat(4097));
        cases.push((long, "is 4097 bytes"));
        let mut at_bound = relay_json();
        at_bound[key] = serde_json::json!("x".repeat(4096));
        assert!(
            serde_json::from_value::<RelayedDecision>(at_bound).is_ok(),
            "{key}"
        );
    }
    for (value, expected) in cases {
        let error = serde_json::from_value::<RelayedDecision>(value.clone())
            .expect_err(&format!("{value} must be refused"))
            .to_string();
        assert!(error.contains("relayed_by"), "{error}");
        assert!(error.contains(expected), "{value}: {error}");
    }
}

/// DISCRIMINATOR for "absent, not null": a record without the key reads as no relay, and a
/// record whose key is `null` is refused rather than read as no relay.
#[test]
fn an_approval_record_without_a_relay_omits_the_key_and_refuses_null() {
    let mut approval = serde_json::json!({
        "gate": "definition",
        "actor": "Ada",
        "timestamp": 1,
        "digest": "d",
        "note": null,
    });
    let read: ApprovalRecord = serde_json::from_value(approval.clone()).unwrap();
    assert!(read.relayed_by.is_none());
    assert!(
        serde_json::to_value(&read)
            .unwrap()
            .get("relayed_by")
            .is_none()
    );

    approval["relayed_by"] = serde_json::Value::Null;
    let error = serde_json::from_value::<ApprovalRecord>(approval)
        .unwrap_err()
        .to_string();
    assert!(error.contains("relayed_by must be an object"), "{error}");
}

/// DISCRIMINATOR for Leif's "Keep it out": the finalization review digest is the same with and
/// without a relay, and for a review without one it is exactly the digest of its bytes, so
/// every review recorded before this change keeps its digest.
#[test]
fn the_review_digest_leaves_the_relay_out_and_keeps_existing_digests() {
    let plain = review_record(None);
    let relayed = review_record(Some(relay("Ada")));
    let plain_bytes_digest = sha256_hex(&serde_json::to_vec(&plain).unwrap());
    assert_eq!(scoped_review_digest(&plain).unwrap(), plain_bytes_digest);
    assert_eq!(scoped_review_digest(&relayed).unwrap(), plain_bytes_digest);
    assert_ne!(
        sha256_hex(&serde_json::to_vec(&relayed).unwrap()),
        plain_bytes_digest,
        "the relay is in the record, only the digest leaves it out"
    );
}

/// DISCRIMINATOR for one decider per record: a library caller cannot attach a relay naming
/// someone other than the approval's actor, or a relay with no actor at all, and nothing is
/// written either way.
#[test]
fn a_relayed_approval_must_name_its_own_actor_as_the_decider() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    let approvals = change_dir(root, &record.id).join("approvals.json");
    let before = fs::read(&approvals).unwrap();

    let other = approve_definition_with_relay(
        root,
        &record.id,
        Some("Ada".into()),
        None,
        false,
        Some(relay("Bob")),
    )
    .unwrap_err();
    assert!(
        other.contains("says `Bob` decided, but --actor records `Ada`"),
        "{other}"
    );
    let missing =
        approve_definition_with_relay(root, &record.id, None, None, false, Some(relay("Ada")))
            .unwrap_err();
    assert!(missing.contains("--actor <who decided>"), "{missing}");
    assert_eq!(fs::read(&approvals).unwrap(), before);
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Draft
    );
}

/// DISCRIMINATOR for the portable pair: both records of a `--portable-5-0-1` approval are one
/// approval, so both carry the relay.
#[test]
fn a_portable_approval_pair_carries_the_relay_on_both_records() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
    append_portable_definition_approval_v501_with_relay(
        root,
        &record,
        Some("Ada".into()),
        None,
        Some(relay("Ada")),
    )
    .unwrap();
    let ledger = load_approvals(root, &record).unwrap();
    let pair: Vec<&ApprovalRecord> = ledger
        .approvals
        .iter()
        .filter(|approval| approval.definition_pair.is_some())
        .collect();
    assert_eq!(pair.len(), 2);
    for approval in pair {
        assert_eq!(approval.relayed_by, Some(relay("Ada")));
    }
}

/// DISCRIMINATOR for CHANGE-13.b: status shows the approval that still stands. One a send-back
/// cancelled is not shown as if it did, and a fresh approval is.
#[test]
fn recorded_claims_skip_an_approval_a_send_back_cancelled() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    approve_definition_with_relay(
        root,
        &record.id,
        Some("Ada".into()),
        None,
        false,
        Some(relay("Ada")),
    )
    .unwrap();
    let approved = load_change(root, &record.id).unwrap();
    assert_eq!(
        recorded_claims(root, &approved).unwrap(),
        vec![RecordedClaim {
            gate: "definition".into(),
            name: "Ada".into(),
            verdict: None,
            relayed_by: Some(relay("Ada")),
        }]
    );

    send_back_change(root, &record.id, "Ada".into(), "re-scope".into()).unwrap();
    let sent_back = load_change(root, &record.id).unwrap();
    assert_eq!(recorded_claims(root, &sent_back).unwrap(), Vec::new());

    approve_definition(root, &record.id, Some("Bea".into()), None).unwrap();
    let renewed = load_change(root, &record.id).unwrap();
    let claims = recorded_claims(root, &renewed).unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].name, "Bea");
    assert_eq!(claims[0].relayed_by, None);
}

/// DISCRIMINATOR for the #900 review (P1): a correction event embeds the approvals it supersedes,
/// and both correction digests hash it. They hash it without the relayed decision, so a
/// correction over a relayed approval keeps the digest an older 6.x binary computes after
/// dropping the key, and every recorded correction keeps its digest.
#[test]
fn correction_digests_leave_embedded_relays_out() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let plain = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap()
    .correction;
    assert_eq!(
        corrected_view_digest(&plain.prior_view_digest, &plain).unwrap(),
        plain.corrected_view_digest,
        "a correction without a relay keeps its recorded digest"
    );
    let mut relayed = plain.clone();
    relayed.superseded_definition_approval.relayed_by = Some(relay("Release reviewer"));
    relayed.superseded_closing_approval.relayed_by = Some(relay("Release reviewer"));
    assert!(
        serde_json::to_string(&relayed)
            .unwrap()
            .contains("relayed_by"),
        "the relay stays in the stored event"
    );
    assert_eq!(
        corrected_view_digest(&relayed.prior_view_digest, &relayed).unwrap(),
        plain.corrected_view_digest
    );
    let change = load_change(root, &record.id).unwrap();
    assert_eq!(
        correction_prefix_digest(&change, std::slice::from_ref(&relayed)).unwrap(),
        correction_prefix_digest(&change, std::slice::from_ref(&plain)).unwrap()
    );
}

/// DISCRIMINATOR for the #900 review (P2): one record names one decider on read as on write. A
/// hand-edited relay naming someone other than the approval's actor is refused, naming the
/// active change's recovery, `repair-relays` (#904), which `repair_relayed_decisions` then runs.
#[test]
fn a_read_relay_must_name_its_own_record_decider() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    approve_definition_with_relay(
        root,
        &record.id,
        Some("Ada".into()),
        None,
        false,
        Some(relay("Ada")),
    )
    .unwrap();
    let path = change_dir(root, &record.id).join("approvals.json");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.replacen("\"decided_by\": \"Ada\"", "\"decided_by\": \"Mallory\"", 1),
    )
    .unwrap();
    let record = load_change(root, &record.id).unwrap();
    let error = load_approvals(root, &record).unwrap_err();
    assert!(error.contains("says `Mallory` decided"), "{error}");
    assert!(
        error.contains(&format!("specsync change repair-relays -- '{}'", record.id)),
        "{error}"
    );
    assert!(!error.contains("git checkout HEAD"), "{error}");
    assert!(recorded_claims(root, &record).is_err());

    let report = repair_relayed_decisions(root, &record.id).unwrap();
    assert_eq!(report.repairs.len(), 1, "{report:?}");
    assert_eq!(report.repairs[0].action, "removed", "{report:?}");
    assert_eq!(
        report.repairs[0].removed.as_ref().unwrap()["decided_by"],
        "Mallory"
    );
    let ledger = load_approvals(root, &record).unwrap();
    assert!(
        ledger
            .approvals
            .iter()
            .all(|approval| approval.relayed_by.is_none())
    );
    assert_eq!(ledger.approvals.last().unwrap().actor, "Ada");
}

/// DISCRIMINATOR for #904 matching: a copy restores a damaged relay only when it is the same
/// record in every other field and its own relay reads cleanly, and a clean record is never
/// treated as damaged, so a repair can never swap one record's relay onto another.
#[test]
fn a_relay_is_put_back_only_from_a_clean_copy_of_the_same_record() {
    let clean = serde_json::to_value(review_record(Some(relay("Ada")))).unwrap();
    assert!(!relay_is_damaged(&clean, "reviewer"));
    assert!(!relay_is_damaged(
        &serde_json::to_value(review_record(None)).unwrap(),
        "reviewer"
    ));

    let mut wrong_decider = clean.clone();
    wrong_decider["relayed_by"]["decided_by"] = serde_json::json!("Mallory");
    assert!(relay_is_damaged(&wrong_decider, "reviewer"));
    let mut null_relay = clean.clone();
    null_relay["relayed_by"] = serde_json::Value::Null;
    assert!(relay_is_damaged(&null_relay, "reviewer"));

    assert_eq!(
        original_relay_in(&wrong_decider, Some(&clean), "reviewer"),
        Some(Some(relay_json()))
    );
    let plain = serde_json::to_value(review_record(None)).unwrap();
    assert_eq!(
        original_relay_in(&null_relay, Some(&plain), "reviewer"),
        Some(None),
        "an original without a relay puts back no relay"
    );
    let mut other_verdict = clean.clone();
    other_verdict["verdict"] = serde_json::json!("block");
    assert_eq!(
        original_relay_in(&wrong_decider, Some(&other_verdict), "reviewer"),
        None,
        "a different record is no copy, even with a clean relay"
    );
    assert_eq!(
        original_relay_in(&wrong_decider, Some(&null_relay), "reviewer"),
        None,
        "a copy whose own relay is damaged is no copy"
    );
    assert_eq!(original_relay_in(&wrong_decider, None, "reviewer"), None);
}

/// DISCRIMINATOR for the #900 review: a library caller that builds `RelayedDecision` directly
/// cannot write a blank or oversized field that the next read would refuse.
#[test]
fn the_write_apis_validate_every_relay_field() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    let approvals = change_dir(root, &record.id).join("approvals.json");
    let before = fs::read(&approvals).unwrap();
    let mut blank = relay("Ada");
    blank.words = "  ".into();
    let mut long = relay("Ada");
    long.recorded_by = "x".repeat(4097);
    for (relay, expected) in [
        (blank, "relayed_by `words` must not be blank"),
        (long, "relayed_by `recorded_by` is 4097 bytes"),
    ] {
        let error = approve_definition_with_relay(
            root,
            &record.id,
            Some("Ada".into()),
            None,
            false,
            Some(relay),
        )
        .unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    assert_eq!(fs::read(&approvals).unwrap(), before);
}

/// DISCRIMINATOR for the #900 review: every review attempt's relay must name that attempt's
/// reviewer, not only the latest projection's.
#[test]
fn every_review_attempt_relay_must_name_its_reviewer() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    let workspace = change_dir(root, &record.id);
    let mut latest = review_record(Some(relay("Ada")));
    latest.change_id = record.id.clone();
    let mut earlier = latest.clone();
    earlier.verdict = ScopedReviewVerdict::Block;
    earlier.relayed_by = Some(relay("Mallory"));
    fs::write(
        workspace.join(SCOPED_REVIEW_FILE),
        json_content(&latest).unwrap(),
    )
    .unwrap();
    let attempts = ScopedReviewAttemptLedger {
        schema_version: 1,
        reviews: vec![earlier, latest],
    };
    fs::write(
        workspace.join(SCOPED_REVIEW_ATTEMPTS_FILE),
        json_content(&attempts).unwrap(),
    )
    .unwrap();
    let error = load_scoped_review(root, &record).unwrap_err();
    assert!(error.contains("says `Mallory` decided"), "{error}");
    assert!(error.contains(SCOPED_REVIEW_ATTEMPTS_FILE), "{error}");
}

/// DISCRIMINATOR for the #900 review: an older 6.x binary that records the next attempt drops
/// `relayed_by` from the earlier ones, and that rewrite is not read as removed evidence. Any
/// other edit to a committed attempt still is.
#[test]
fn review_history_comparisons_ignore_relays_but_not_other_edits() {
    let relayed = review_record(Some(relay("Ada")));
    let dropped = review_record(None);
    let mut next = review_record(None);
    next.timestamp += 60;
    let ledger = |reviews: Vec<ScopedReviewRecord>| ScopedReviewAttemptLedger {
        schema_version: 1,
        reviews,
    };
    let path = ".specsync/changes/add-auth/review-attempts.json".to_string();
    let previous = (path.clone(), ledger(vec![relayed.clone()]));
    let rewritten_by_older = (path.clone(), ledger(vec![dropped.clone(), next.clone()]));
    validate_scoped_review_history_transition(Some(&previous), Some(&rewritten_by_older), false)
        .unwrap();
    validate_scoped_review_history_transition(
        Some(&previous),
        Some(&(path.clone(), ledger(vec![dropped]))),
        false,
    )
    .unwrap();
    let mut edited = relayed;
    edited.verdict = ScopedReviewVerdict::Block;
    let error = validate_scoped_review_history_transition(
        Some(&previous),
        Some(&(path, ledger(vec![edited, next]))),
        false,
    )
    .unwrap_err();
    assert!(
        error.contains("removed or rewrote committed evidence"),
        "{error}"
    );
}

/// DISCRIMINATOR for the #995 review (P1): a change id is any portable path component, so it can
/// hold spaces and shell syntax or start with `-`. Every recovery command a relay refusal names
/// renders the id and the path as single shell words after `--`, so copying it runs exactly that
/// command and the id cannot be read as an option.
#[test]
fn relay_recoveries_quote_the_change_id_and_path_after_double_dash() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_section_only_current_record(root, RELAY_DELTA);
    record.id = "-x a;$(y)".into();
    let active = with_relayed_read_recovery(
        root,
        &record,
        &root.join(".specsync/changes/-x a;$(y)/review.json"),
        "invalid relayed_by".into(),
    );
    assert!(
        active.contains("`specsync change repair-relays -- '-x a;$(y)'`"),
        "{active}"
    );
    assert!(!active.contains("repair-relays -x"), "{active}");

    record.state = ChangeState::Archived;
    let archived = with_relayed_read_recovery(
        root,
        &record,
        &root.join(".specsync/archive/changes/2026-11-06--x a;$(y)/review.json"),
        "invalid relayed_by".into(),
    );
    assert!(
        archived.contains(
            "`git checkout HEAD -- '.specsync/archive/changes/2026-11-06--x a;$(y)/review.json'`"
        ),
        "{archived}"
    );
    let it = "it's";
    assert_eq!(shell_quote(it), "'it'\\''s'");
}

/// DISCRIMINATOR for the second #995 review (P2): a committed copy is "absent" only when Git says
/// so: no work tree, an unborn `HEAD`, or no such path in `HEAD`'s tree. A `HEAD` that exists but
/// does not resolve is an error, so `repair-relays` never removes a relay `HEAD` may still hold.
#[test]
fn only_git_saying_so_counts_as_no_committed_copy() {
    let git = |root: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("ledger.json"), "{}\n").unwrap();
    assert_eq!(committed_copy_at_head(root, "ledger.json").unwrap(), None);

    git(root, &["init", "-q"]);
    assert_eq!(
        committed_copy_at_head(root, "ledger.json").unwrap(),
        None,
        "an unborn HEAD holds no copy"
    );

    git(root, &["add", "ledger.json"]);
    git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-q",
            "-m",
            "ledger",
        ],
    );
    assert_eq!(
        committed_copy_at_head(root, "ledger.json").unwrap(),
        Some(b"{}\n".to_vec())
    );
    assert_eq!(committed_copy_at_head(root, "missing.json").unwrap(), None);

    let reference = git(root, &["symbolic-ref", "HEAD"]).trim().to_string();
    fs::write(root.join(".git").join(reference), "not a commit\n").unwrap();
    let error = committed_copy_at_head(root, "ledger.json").unwrap_err();
    assert!(error.contains("does not resolve"), "{error}");
}

// G-relay-compare (#926, #927, #971): a relayed decision is covered by no digest and read by no
// gate, so an older 6.x binary that drops the unknown key must get the same answers, and so must
// an edit to a relay. Three comparisons still saw it.

fn without_relayed_by(value: &serde_json::Value) -> serde_json::Value {
    let mut value = value.clone();
    value.as_object_mut().unwrap().remove("relayed_by");
    value
}

/// DISCRIMINATOR for #926: the approval-succession check reads a ledger an older binary rewrote
/// without `relayed_by` (on the approvals and on the superseded approval) as a lawful successor,
/// and any other edit to an earlier approval still is not one.
#[test]
fn approval_succession_ignores_a_relay_an_older_binary_dropped() {
    let definition = serde_json::json!({
        "gate": "definition", "actor": "Ada", "digest": "d1", "relayed_by": relay_json(),
    });
    let acceptance = serde_json::json!({
        "gate": "acceptance", "actor": "Ada", "digest": "a1", "relayed_by": relay_json(),
    });
    let earlier = serde_json::json!({ "approvals": [definition, acceptance], "reopenings": [] });
    let earlier = serde_json::to_vec(&earlier).unwrap();
    let successor = |approvals: Vec<serde_json::Value>, superseded: serde_json::Value| {
        serde_json::to_vec(&serde_json::json!({
            "approvals": approvals,
            "reopenings": [{ "actor": "Ada", "reason": "fix", "superseded_approval": superseded }],
        }))
        .unwrap()
    };

    let kept = successor(
        vec![definition.clone(), acceptance.clone()],
        acceptance.clone(),
    );
    assert_eq!(
        ledger_succession(Some(&earlier), &kept),
        LedgerSuccession::LawfulSuccessor
    );
    let dropped = successor(
        vec![
            without_relayed_by(&definition),
            without_relayed_by(&acceptance),
        ],
        without_relayed_by(&acceptance),
    );
    assert_eq!(
        ledger_succession(Some(&earlier), &dropped),
        LedgerSuccession::LawfulSuccessor,
        "an older binary's reopen drops relayed_by and is still the successor"
    );

    let mut edited = without_relayed_by(&definition);
    edited["actor"] = serde_json::json!("Mallory");
    let edited = successor(
        vec![edited, without_relayed_by(&acceptance)],
        without_relayed_by(&acceptance),
    );
    assert_eq!(
        ledger_succession(Some(&earlier), &edited),
        LedgerSuccession::NotASuccessor
    );
    let mut superseded = without_relayed_by(&acceptance);
    superseded["digest"] = serde_json::json!("a2");
    let wrong_superseded = successor(
        vec![
            without_relayed_by(&definition),
            without_relayed_by(&acceptance),
        ],
        superseded,
    );
    assert_eq!(
        ledger_succession(Some(&earlier), &wrong_superseded),
        LedgerSuccession::NotASuccessor
    );
}

/// DISCRIMINATOR for #927: the definition digest hashes the correction ledger without the relays
/// its embedded approvals carry, so editing or dropping one changes no gate, and any other edit
/// to a correction still changes the digest.
#[test]
fn the_definition_digest_leaves_relays_in_corrections_out() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let plain = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap()
    .correction;
    let mut relayed = plain.clone();
    relayed.superseded_definition_approval.relayed_by = Some(relay("Release reviewer"));
    relayed.superseded_closing_approval.relayed_by = Some(relay("Release reviewer"));
    let change = load_change(root, &record.id).unwrap();
    let snapshot = DefinitionArtifactSnapshot {
        entries: Vec::new(),
        correction_mode: Some(0o100644),
    };
    let digest = |correction: &CorrectionRecord| {
        definition_digest_from_snapshot(&change, b"{}", std::slice::from_ref(correction), &snapshot)
            .unwrap()
    };
    assert_eq!(digest(&relayed), digest(&plain));
    let mut other = relayed.clone();
    other.superseded_closing_approval.relayed_by = Some(relay("Someone else"));
    assert_eq!(
        digest(&other),
        digest(&plain),
        "editing a relay changes nothing"
    );
    let mut edited = plain.clone();
    edited.reason = "A different reason".into();
    assert_ne!(digest(&edited), digest(&plain));
}

/// DISCRIMINATOR for #971: the trusted correction-history comparison reads a correction ledger an
/// older binary rewrote without the embedded relays as the same history, and any other edit to a
/// committed correction is still divergence.
#[test]
fn trusted_correction_history_ignores_a_relay_an_older_binary_dropped() {
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
    let corrections_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let mut ledger: CorrectionLedger =
        serde_json::from_str(&fs::read_to_string(&corrections_path).unwrap()).unwrap();
    let decider = ledger.corrections[0]
        .superseded_closing_approval
        .actor
        .clone();
    ledger.corrections[0].superseded_closing_approval.relayed_by = Some(relay(&decider));
    fs::write(&corrections_path, json_content(&ledger).unwrap()).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "accept a correction whose closing approval carries a relay",
    ]);
    let committed = fs::read_to_string(&corrections_path).unwrap();
    assert!(committed.contains("relayed_by"));
    let head = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let directory = format!("{CHANGES_PATH}/{}", record.id);
    assert!(
        closing_authenticated_correction_anchor(root, &head, &directory, &record.id)
            .unwrap()
            .is_some(),
        "the committed correction is a trusted anchor, so the comparison below is not vacuous"
    );

    let mut older = ledger.clone();
    older.corrections[0].superseded_closing_approval.relayed_by = None;
    fs::write(&corrections_path, json_content(&older).unwrap()).unwrap();
    let current = load_change(root, &record.id).unwrap();
    effective_change_definition(root, &current)
        .expect("an older binary's rewrite without the relay is the same history");

    let mut edited = older.clone();
    edited.corrections[0].reason = "A rewritten reason".into();
    fs::write(&corrections_path, json_content(&edited).unwrap()).unwrap();
    let error = effective_change_definition(root, &current).unwrap_err();
    assert!(
        error.contains("correction history divergence detected"),
        "{error}"
    );
}

// PR #1014 review regressions. All actors and acceptance records below are private TEST fixtures.

#[test]
fn pr1014_succession_preserves_a_reopenings_top_level_relay_key() {
    let closing = serde_json::json!({"gate": "acceptance", "actor": "Ada", "digest": "a1"});
    let reopening = serde_json::json!({
        "actor": "Ada", "reason": "first reopen", "superseded_approval": closing,
        "relayed_by": "unknown reopening field, not an approval relay",
    });
    let earlier = serde_json::json!({"approvals": [closing], "reopenings": [reopening]});
    let mut current = earlier.clone();
    current["reopenings"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "actor": "Ada", "reason": "second reopen", "superseded_approval": closing,
        }));
    let bytes = |value: &serde_json::Value| serde_json::to_vec(value).unwrap();
    assert_eq!(
        ledger_succession(Some(&bytes(&earlier)), &bytes(&current)),
        LedgerSuccession::LawfulSuccessor,
        "the unchanged committed prefix is a valid control"
    );
    for remove in [false, true] {
        let mut tampered = current.clone();
        let first = tampered["reopenings"][0].as_object_mut().unwrap();
        if remove {
            first.remove("relayed_by");
        } else {
            first.insert(
                "relayed_by".into(),
                serde_json::json!("rewritten unknown field"),
            );
        }
        assert_eq!(
            ledger_succession(Some(&bytes(&earlier)), &bytes(&tampered)),
            LedgerSuccession::NotASuccessor,
            "a reopening's own unknown key is not an approval relay (remove={remove})"
        );
    }
}

#[test]
fn pr1014_accepted_evidence_keys_ignore_only_approval_relays() {
    let approval = serde_json::json!({
        "gate": "acceptance", "actor": "Ada", "timestamp": 1, "digest": "a1",
        "note": null, "relayed_by": relay_json(), "unknown_approval_field": "keep",
    });
    let ledger = serde_json::json!({
        "approvals": [approval],
        "reopenings": [{"superseded_approval": approval, "relayed_by": "keep reopening field"}],
        "unknown_ledger_field": "keep",
    });
    let key = |value: &serde_json::Value| {
        accepted_evidence_key(
            b"state",
            b"verification",
            &serde_json::to_vec(value).unwrap(),
        )
    };
    let mut dropped = ledger.clone();
    dropped["approvals"][0]
        .as_object_mut()
        .unwrap()
        .remove("relayed_by");
    dropped["reopenings"][0]["superseded_approval"]
        .as_object_mut()
        .unwrap()
        .remove("relayed_by");
    assert_eq!(
        key(&ledger),
        key(&dropped),
        "an older binary's relay-free ledger is the same evidence"
    );
    let mut edited_relay = ledger.clone();
    edited_relay["approvals"][0]["relayed_by"]["words"] =
        serde_json::json!("different recorded words");
    assert_eq!(key(&ledger), key(&edited_relay));
    for path in ["actor", "unknown_approval_field"] {
        let mut edited = dropped.clone();
        edited["approvals"][0][path] = serde_json::json!("changed");
        assert_ne!(
            key(&dropped),
            key(&edited),
            "approval field {path} remains significant"
        );
    }
    let mut edited = dropped.clone();
    edited["reopenings"][0]["relayed_by"] = serde_json::json!("changed");
    assert_ne!(key(&dropped), key(&edited));
    edited = dropped.clone();
    edited["unknown_ledger_field"] = serde_json::json!("changed");
    assert_ne!(key(&dropped), key(&edited));
    assert_ne!(
        accepted_evidence_key(b"state", b"verification", b"not json"),
        accepted_evidence_key(b"state", b"verification", b"also not json"),
        "invalid ledgers must not collapse to one empty projection"
    );
}

#[test]
fn pr1014_accepted_anchor_ignores_a_dropped_approval_relay_but_keeps_other_fields() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "TEST fixture"]);
    git(&["config", "core.autocrlf", "false"]);
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let directory = format!("{CHANGES_PATH}/{}", record.id);
    let approvals_path = format!("{directory}/approvals.json");
    let verification_path = format!("{directory}/verification.json");
    let state_path = format!("{directory}/state.json");
    let mut ledger: ApprovalLedger =
        serde_json::from_slice(&fs::read(root.join(&approvals_path)).unwrap()).unwrap();
    for approval in &mut ledger.approvals {
        approval.relayed_by = Some(relay(&approval.actor));
    }
    fs::write(root.join(&approvals_path), json_content(&ledger).unwrap()).unwrap();
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "TEST accepted ledger with valid informational relays",
    ]);
    let anchor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let verification = fs::read(root.join(&verification_path)).unwrap();
    let original = fs::read(root.join(&approvals_path)).unwrap();
    let eligible = |approvals: &[u8]| {
        let mut eligible = BTreeMap::new();
        consider_accepted_evidence_anchor(
            root,
            &record,
            &anchor,
            &state_path,
            &verification_path,
            &approvals_path,
            &verification,
            approvals,
            &mut eligible,
        );
        eligible
    };
    assert_eq!(
        eligible(&original).len(),
        1,
        "the committed accepted anchor must be non-vacuously eligible"
    );
    for approval in &mut ledger.approvals {
        approval.relayed_by = None;
    }
    let dropped = json_content(&ledger).unwrap();
    assert_eq!(
        eligible(dropped.as_bytes()).len(),
        1,
        "dropping only approval relays keeps the anchor"
    );
    ledger.approvals[0].actor = "different TEST actor".into();
    assert!(
        eligible(json_content(&ledger).unwrap().as_bytes()).is_empty(),
        "a non-relay edit is still refused"
    );
    assert!(
        eligible(b"not JSON").is_empty(),
        "an invalid ledger is not an eligible anchor"
    );
}

#[test]
fn pr1014_correction_read_refuses_each_embedded_relay_decider_mismatch() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "TEST correction actor".into(),
        "TEST classification correction".into(),
    )
    .unwrap();
    let record = load_change(root, &record.id).unwrap();
    let path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let mut ledger: CorrectionLedger = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let definition = &mut ledger.corrections[0].superseded_definition_approval;
    definition.relayed_by = Some(relay(&definition.actor));
    let closing = &mut ledger.corrections[0].superseded_closing_approval;
    closing.relayed_by = Some(relay(&closing.actor));
    fs::write(&path, json_content(&ledger).unwrap()).unwrap();
    effective_change_definition(root, &record).expect("matching embedded deciders are valid");
    for definition in [true, false] {
        let mut damaged = ledger.clone();
        let correction = &mut damaged.corrections[0];
        let approval = if definition {
            &mut correction.superseded_definition_approval
        } else {
            &mut correction.superseded_closing_approval
        };
        approval.relayed_by.as_mut().unwrap().decided_by = "different TEST decider".into();
        fs::write(&path, json_content(&damaged).unwrap()).unwrap();
        let error = effective_change_definition(root, &record)
            .expect_err("an embedded relay must name its own approval's actor");
        assert!(
            error.contains("recorded relayed_by does not match its record"),
            "{error}"
        );
    }
    fs::write(&path, json_content(&ledger).unwrap()).unwrap();
    effective_change_definition(root, &record)
        .expect("restoring valid relays recovers the same record");
}

// Independent review of 43a402bc: unknown numbers must not lose precision in a relay projection.
#[test]
fn pr1014_numeric_projection_keeps_unknown_decimal_values_distinct() {
    let left = br#"{"schema_version":1,"approvals":[],"reopenings":[],"unknown_ledger_field":1.000000000000000000001}"#;
    let right = br#"{"schema_version":1,"approvals":[],"reopenings":[],"unknown_ledger_field":1.000000000000000000002}"#;
    assert_ne!(left, right, "the raw evidence differs");
    serde_json::from_slice::<ApprovalLedger>(left)
        .expect("the real typed ledger accepts this unknown field");
    serde_json::from_slice::<ApprovalLedger>(right)
        .expect("the changed unknown field is still decodable");
    assert_ne!(
        relay_blind_approval_ledger_bytes(left).unwrap(),
        relay_blind_approval_ledger_bytes(right).unwrap(),
        "an unknown numeric-field edit must not collapse during relay projection"
    );
    assert_ne!(
        accepted_evidence_key(b"state", b"verification", left),
        accepted_evidence_key(b"state", b"verification", right),
        "distinct non-relay evidence needs distinct anchor keys"
    );
}

#[test]
fn pr1014_numeric_edit_cannot_reuse_a_reachable_accepted_anchor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "TEST numeric fixture"]);
    git(&["config", "core.autocrlf", "false"]);
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let directory = format!("{CHANGES_PATH}/{}", record.id);
    let state_path = format!("{directory}/state.json");
    let verification_path = format!("{directory}/verification.json");
    let approvals_path = format!("{directory}/approvals.json");
    let serialized = fs::read_to_string(root.join(&approvals_path)).unwrap();
    let with_number =
        |number: &str| format!("{{\"unknown_ledger_field\":{number},{}", &serialized[1..]);
    let original = with_number("1.000000000000000000001");
    let tampered = with_number("1.000000000000000000002");
    assert!(load_approvals(root, &record).is_ok());
    serde_json::from_str::<ApprovalLedger>(&original).expect("valid original wire ledger");
    serde_json::from_str::<ApprovalLedger>(&tampered)
        .expect("the typed decoder does not block the attack");
    fs::write(root.join(&approvals_path), &original).unwrap();
    assert!(
        load_approvals(root, &record).is_ok(),
        "actual ledger reader accepts the unknown numeric field"
    );
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "TEST reachable accepted anchor with exact unknown decimal",
    ]);
    let anchor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let verification = fs::read(root.join(&verification_path)).unwrap();
    let eligible = |wire: &[u8]| {
        let mut eligible = BTreeMap::new();
        consider_accepted_evidence_anchor(
            root,
            &record,
            &anchor,
            &state_path,
            &verification_path,
            &approvals_path,
            &verification,
            wire,
            &mut eligible,
        );
        eligible
    };
    assert_eq!(
        eligible(original.as_bytes()).len(),
        1,
        "the committed original anchor is eligible"
    );
    fs::write(root.join(&approvals_path), &tampered).unwrap();
    assert!(
        load_approvals(root, &record).is_ok(),
        "the changed unknown field reaches anchor comparison"
    );
    assert!(
        eligible(tampered.as_bytes()).is_empty(),
        "a non-relay numeric edit must reject the historical anchor"
    );
    assert_eq!(
        eligible(original.as_bytes()).len(),
        1,
        "the original wire evidence remains eligible"
    );
}

#[test]
fn pr1014_numeric_wire_rewrites_ignore_only_the_allowed_relay_positions() {
    let relayed = br#"{
        "approvals":[{"gate":"definition","actor":"Ada","timestamp":1,"digest":"d","note":null,
          "relayed_by":{"decided_by":"Ada","words":"yes","where":"TEST","recorded_by":"TEST"}}],
        "reopenings":[{"superseded_approval":{"actor":"Ada","relayed_by":{"decided_by":"Ada","words":"yes","where":"TEST","recorded_by":"TEST"}},"relayed_by":"reopening data"}],
        "unknown":{"number":10e-2,"text":"a \"quote\" and }[,\\","relayed_by":{"nested":"must remain"}}
    }"#;
    let rewritten = br#"{"unknown":{"relayed_by":{"nested":"must remain"},"text":"\u0061 \"quote\" and }[,\\","number":10e-2},
      "reopenings":[{"relayed_by":"reopening data","superseded_approval":{"actor":"Ada"}}],
      "approvals":[{"note":null,"digest":"d","timestamp":1,"actor":"Ada","gate":"definition"}]}"#;
    let first = relay_blind_approval_ledger_bytes(relayed).expect("valid relayed wire ledger");
    assert_eq!(first, relay_blind_approval_ledger_bytes(rewritten).unwrap(),
        "whitespace, key ordering and equivalent string escapes do not change evidence");
    assert_eq!(accepted_evidence_key(b"s", b"v", relayed), accepted_evidence_key(b"s", b"v", rewritten));
    let tampered = String::from_utf8(rewritten.to_vec()).unwrap().replace("must remain", "edited nested field");
    assert_ne!(first, relay_blind_approval_ledger_bytes(tampered.as_bytes()).unwrap(),
        "an unknown nested relayed_by is not an approval relay");
    let tampered = String::from_utf8(rewritten.to_vec()).unwrap().replace("reopening data", "edited reopening field");
    assert_ne!(first, relay_blind_approval_ledger_bytes(tampered.as_bytes()).unwrap());
}

#[test]
fn pr1014_numeric_wire_projection_refuses_malformed_or_trailing_input() {
    for input in [
        br#"{"approvals":[],"x":"bad\q"}"#.as_slice(),
        br#"{"approvals":[],"x":[1,2}"#.as_slice(),
        br#"{"approvals":[],}"#.as_slice(),
        br#"{"approvals":[]} true"#.as_slice(),
        br#"{"approvals":[],"x":01}"#.as_slice(),
        br#"{"approvals":[],"x":1e+}"#.as_slice(),
        br#"[1,2,3]"#.as_slice(),
    ] {
        assert!(relay_blind_approval_ledger_bytes(input).is_none(), "invalid ledger: {input:?}");
    }
    assert!(relay_blind_approval_ledger_bytes(br#"{"approvals":[],"x":"quoted }[,\\\" delimiters"}"#).is_some());
}

#[test]
fn pr1014_numeric_wire_projection_preserves_exact_scalars_and_duplicate_unknown_fields() {
    let project = |value: &str| {
        relay_blind_approval_ledger_bytes(format!("{{\"approvals\":[],\"unknown\":{value}}}").as_bytes())
            .expect("a valid unknown scalar accepted by the ledger decoder keeps its exact token")
    };
    let values = ["1.000000000000000000001", "1.000000000000000000002", "1e400", "2e400",
        "18446744073709551616", "18446744073709551617", "\"1e400\"", "true", "null"];
    for (index, left) in values.iter().enumerate() {
        for right in &values[index + 1..] {
            assert_ne!(project(left), project(right), "distinct scalar values/types: {left} vs {right}");
        }
    }
    let left = br#"{"approvals":[],"unknown":1.000000000000000000001,"unknown":0}"#;
    let right = br#"{"approvals":[],"unknown":1.000000000000000000002,"unknown":0}"#;
    serde_json::from_slice::<ApprovalLedger>(left).unwrap();
    serde_json::from_slice::<ApprovalLedger>(right).unwrap();
    assert_ne!(relay_blind_approval_ledger_bytes(left).unwrap(), relay_blind_approval_ledger_bytes(right).unwrap(),
        "a duplicate unknown member must not erase an earlier non-relay field edit");
}

// Succession precision review: these are private TEST generations and unauthenticated TEST actors.
// Build both generations through native transitions before injecting raw unknown decimal tokens.
fn pr1014_succession_precision_generations(
    root: &Path,
) -> (ChangeRecord, serde_json::Value, serde_json::Value) {
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "TEST succession fixture"]);
    git(&["config", "core.autocrlf", "false"]);
    write_lifecycle_test_policy(root);
    let record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "TEST original succession definition"]);
    let accepted = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "TEST first accepted generation"]);

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &accepted.id,
        "TEST reopening actor".into(),
        "TEST first changed delivery inputs".into(),
    )
    .expect("a real stale-input reopen creates the first audit");
    verify_change(root, &reopened.change.id)
        .expect("verify the reopened TEST generation without restarting implementation");
    let accepted = accept_change(
        root,
        &reopened.change.id,
        Some("TEST closing actor".into()),
        None,
    )
    .expect("accept the verified reopened TEST generation");
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "TEST accepted generation with one completed reopen",
    ]);
    let path = change_dir(root, &accepted.id).join("approvals.json");
    let earlier_wire = fs::read(&path).unwrap();
    let earlier_typed = load_approvals(root, &accepted).expect("the earlier native ledger reads");
    assert_eq!(earlier_typed.reopenings.len(), 1);

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { true } // next TEST generation\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &accepted.id,
        "TEST reopening actor".into(),
        "TEST second changed delivery inputs".into(),
    )
    .expect("a second real reopen extends that accepted generation");
    let current_wire = fs::read(&path).unwrap();
    let current_typed =
        load_approvals(root, &reopened.change).expect("the actual successor ledger reads");
    assert_eq!(
        current_typed.reopenings.len(),
        earlier_typed.reopenings.len() + 1
    );
    assert_eq!(
        ledger_succession(Some(&earlier_wire), &current_wire),
        LedgerSuccession::LawfulSuccessor,
        "the unmodified native generations must be a lawful control before any unknown-field injection"
    );
    (
        reopened.change,
        serde_json::from_slice(&earlier_wire).unwrap(),
        serde_json::from_slice(&current_wire).unwrap(),
    )
}

#[derive(Clone, Copy, Debug)]
enum Pr1014SuccessionPrecisionField {
    ApprovalPrefix,
    ReopeningPrefix,
    SupersededApproval,
}

fn pr1014_succession_precision_discriminator(field: Pr1014SuccessionPrecisionField) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (record, mut earlier, mut current) = pr1014_succession_precision_generations(root);
    let terminal = earlier["approvals"]
        .as_array()
        .unwrap()
        .iter()
        .rposition(|entry| matches!(entry["gate"].as_str(), Some("acceptance" | "finalization")))
        .expect("the genuinely accepted earlier generation has a closing approval");
    let first_new_reopen = earlier["reopenings"].as_array().unwrap().len();
    let one = serde_json::json!("__PR1014_EXACT_ONE__");
    match field {
        Pr1014SuccessionPrecisionField::ApprovalPrefix => {
            earlier["approvals"][0]["unknown_precision"] = one.clone();
            current["approvals"][0]["unknown_precision"] = one.clone();
        }
        Pr1014SuccessionPrecisionField::ReopeningPrefix => {
            earlier["reopenings"][0]["unknown_precision"] = one.clone();
            current["reopenings"][0]["unknown_precision"] = one.clone();
        }
        Pr1014SuccessionPrecisionField::SupersededApproval => {
            earlier["approvals"][terminal]["unknown_precision"] = one.clone();
            current["approvals"][terminal]["unknown_precision"] = one.clone();
            current["reopenings"][first_new_reopen]["superseded_approval"]["unknown_precision"] =
                one;
        }
    }
    // Substitute numeric tokens after JSON serialization, never through a Value/f64 conversion.
    let wire = |value: &serde_json::Value| {
        serde_json::to_string(value)
            .unwrap()
            .replace("\"__PR1014_EXACT_ONE__\"", "1.000000000000000000001")
            .replace("\"__PR1014_EXACT_TWO__\"", "1.000000000000000000002")
            .into_bytes()
    };
    let earlier_wire = wire(&earlier);
    let control_wire = wire(&current);
    let path = change_dir(root, &record.id).join("approvals.json");
    for (label, bytes) in [
        ("earlier", &earlier_wire),
        ("unchanged successor", &control_wire),
    ] {
        fs::write(&path, bytes).unwrap();
        load_approvals(root, &record).unwrap_or_else(|error| {
            panic!("{label} raw numeric ledger must reach comparison: {error}")
        });
    }
    assert_eq!(
        ledger_succession(Some(&earlier_wire), &control_wire),
        LedgerSuccession::LawfulSuccessor,
        "the same exact unknown token is a lawful successor before the {field:?} edit"
    );
    let two = serde_json::json!("__PR1014_EXACT_TWO__");
    match field {
        Pr1014SuccessionPrecisionField::ApprovalPrefix => {
            current["approvals"][0]["unknown_precision"] = two
        }
        Pr1014SuccessionPrecisionField::ReopeningPrefix => {
            current["reopenings"][0]["unknown_precision"] = two
        }
        Pr1014SuccessionPrecisionField::SupersededApproval => {
            current["reopenings"][first_new_reopen]["superseded_approval"]["unknown_precision"] =
                two
        }
    }
    let altered_wire = wire(&current);
    assert_ne!(
        altered_wire, control_wire,
        "the exact non-relay decimal token changed"
    );
    fs::write(&path, &altered_wire).unwrap();
    load_approvals(root, &record)
        .expect("the altered unknown token is valid for the actual typed reader");
    assert_eq!(
        ledger_succession(Some(&earlier_wire), &altered_wire),
        LedgerSuccession::NotASuccessor,
        "an exact unknown decimal edit at {field:?} must not be erased by f64 rounding"
    );
}

#[test]
fn pr1014_succession_precision_rejects_an_approval_prefix_edit() {
    pr1014_succession_precision_discriminator(Pr1014SuccessionPrecisionField::ApprovalPrefix);
}

#[test]
fn pr1014_succession_precision_rejects_a_reopening_prefix_edit() {
    pr1014_succession_precision_discriminator(Pr1014SuccessionPrecisionField::ReopeningPrefix);
}

#[test]
fn pr1014_succession_precision_rejects_a_superseded_approval_edit() {
    pr1014_succession_precision_discriminator(Pr1014SuccessionPrecisionField::SupersededApproval);
}

// ─── #1064, REQ-change-158: the relay gate's own rules ──────────────────────────────────────

/// DISCRIMINATOR for REQ-change-158: the history may raise a workflow-v2 change to the
/// relay-gated version with its origin kept at 2, and nothing else moves the version.
#[test]
fn only_the_relay_gate_raise_moves_the_workflow_version() {
    let lawful = [
        ((1, None), (1, None)),
        ((1, None), (1, Some(1))),
        ((2, None), (2, Some(2))),
        ((2, Some(2)), (2, Some(2))),
        ((2, Some(2)), (3, Some(2))),
        ((2, None), (3, Some(2))),
        ((3, Some(2)), (3, Some(2))),
    ];
    for (previous, current) in lawful {
        assert!(workflow_version_move_is_lawful(previous, current), "{previous:?} -> {current:?}");
    }
    let unlawful = [
        ((3, Some(2)), (2, Some(2))),
        ((2, Some(2)), (3, Some(3))),
        ((2, Some(2)), (3, None)),
        ((1, Some(1)), (3, Some(2))),
        ((1, None), (2, Some(2))),
        ((2, Some(2)), (1, Some(1))),
        ((3, Some(2)), (4, Some(2))),
        ((2, Some(1)), (3, Some(2))),
    ];
    for (previous, current) in unlawful {
        assert!(!workflow_version_move_is_lawful(previous, current), "{previous:?} -> {current:?}");
    }
}

/// GUARD for REQ-change-158 and REQ-change-079: a relay-gated record is valid only with origin
/// 2, and the digest reads it as the workflow-v2 change it is.
#[test]
fn a_relay_gated_record_needs_origin_2_and_digests_as_workflow_v2() {
    let mut record = minimal_archived_record("gate-it", vec!["README.md".into()]);
    record.workflow_version = RELAY_GATED_WORKFLOW_VERSION;
    record.workflow_origin_version = Some(2);
    assert!(validate_workflow_version_anchor(&record).is_ok());
    assert!(validate_historical_workflow_version_anchor(&record).is_ok());
    for origin in [None, Some(1), Some(3)] {
        record.workflow_origin_version = origin;
        assert!(validate_workflow_version_anchor(&record).is_err(), "{origin:?}");
        assert!(validate_historical_workflow_version_anchor(&record).is_err(), "{origin:?}");
    }
    assert_eq!(digest_workflow_version(RELAY_GATED_WORKFLOW_VERSION), 2);
    assert_eq!(digest_workflow_version(2), 2);
    assert_eq!(digest_workflow_version(1), 1);
    assert_eq!(digest_workflow_version(4), 4);
}

/// DISCRIMINATOR for REQ-change-158 (any rewrite raises): a relay anywhere in a decision ledger,
/// embedded in a correction or in a ledger that no longer parses, counts; a ledger without one
/// does not, and neither does the word in a value.
#[test]
fn a_relay_anywhere_in_a_decision_ledger_gates_the_change() {
    let embedded = serde_json::json!({
        "schema_version": 1,
        "corrections": [{"superseded_definition_approval": {"actor": "Ada", "relayed_by": relay_json()}}],
    });
    assert!(ledger_holds_relay(embedded.to_string().as_bytes()));
    assert!(ledger_holds_relay(br#"{"approvals": [{"relayed_by": {"decided_by": "Ada""#));
    assert!(!ledger_holds_relay(br#"{"approvals": [{"actor": "Ada", "note": "relayed_by is a word here"}]}"#));
    assert!(!ledger_holds_relay(br#"{"reviews": []}"#));

    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path();
    let mut record = minimal_archived_record("gate-it", vec!["README.md".into()]);
    record.workflow_version = 2;
    record.workflow_origin_version = Some(2);
    assert_eq!(relay_gated_record(workspace, &record).workflow_version, 2);
    fs::write(workspace.join(CORRECTIONS_FILE), embedded.to_string()).unwrap();
    assert_eq!(relay_gated_record(workspace, &record).workflow_version, 3);
    fs::remove_file(workspace.join(CORRECTIONS_FILE)).unwrap();
    fs::write(workspace.join("state.json"), r#"{"workflow_version": 3}"#).unwrap();
    assert_eq!(
        relay_gated_record(workspace, &record).workflow_version,
        3,
        "a copy read before the raise never lowers the file it replaces"
    );
    record.workflow_version = 1;
    record.workflow_origin_version = Some(1);
    fs::write(workspace.join(CORRECTIONS_FILE), embedded.to_string()).unwrap();
    assert_eq!(relay_gated_record(workspace, &record).workflow_version, 1);
}

/// A verification record for a synthetic reopening, as a refused attempt records one.
fn synthetic_prior_verification() -> VerificationRecord {
    VerificationRecord {
        timestamp: 0,
        commit: None,
        contract_digest: String::new(),
        execution_digest: None,
        workspace_digest: String::new(),
        acceptance_input_digest: None,
        acceptance_manifest: None,
        semantic_succession: None,
        passed: false,
        commands: Vec::new(),
        requirement_ids: Vec::new(),
        refusal: Some("synthetic".into()),
    }
}

/// Commit everything under `root` in a fresh repository, so `HEAD` holds every ledger.
fn commit_project(root: &Path) {
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-q",
            "-m",
            "project",
        ],
    ] {
        let output = std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// #998 (REQ-change-relay-repair-001): a damaged relay nested in a reopening's
/// `superseded_approval` is put back from the same place in `approvals.json` at `HEAD`, the
/// file's bytes then equal the committed file, and the report names the nested record.
#[test]
fn repair_puts_back_a_damaged_reopening_relay_from_head() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, RELAY_DELTA);
    approve_definition_with_relay(
        root,
        &record.id,
        Some("Ada".into()),
        None,
        false,
        Some(relay("Ada")),
    )
    .unwrap();
    let record = load_change(root, &record.id).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.reopenings.push(ReopenRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: "Ada".into(),
        reason: "Synthetic reopening".into(),
        timestamp: now(),
        from_state: ChangeState::Accepted,
        to_state: ChangeState::Verifying,
        superseded_approval: ledger.approvals.last().unwrap().clone(),
        prior_verification: synthetic_prior_verification(),
        stale_acceptance_input_digest: "0".repeat(64),
        current_acceptance_input_digest: "1".repeat(64),
        stale_evidence_cause: None,
        recorded_review_mismatch_anchor: None,
    });
    let path = change_dir(root, &record.id).join("approvals.json");
    write_json(&path, &ledger).unwrap();
    commit_project(root);
    let committed = fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&committed).unwrap();
    value["reopenings"][0]["superseded_approval"]["relayed_by"]["decided_by"] =
        serde_json::json!("Mallory");
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    let report = repair_relayed_decisions(root, &record.id).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["repairs"].as_array().map(Vec::len), Some(1), "{json:#}");
    let repair = &json["repairs"][0];
    assert_eq!(repair["file"], "approvals.json", "{json:#}");
    assert_eq!(repair["record"], 1, "{json:#}");
    assert_eq!(repair["within"], "superseded_approval", "{json:#}");
    assert_eq!(repair["action"], "put_back", "{json:#}");
    assert_eq!(repair["from"], "HEAD", "{json:#}");
    assert_eq!(fs::read(&path).unwrap(), committed, "the committed bytes are back");
    load_approvals(root, &record).unwrap();
}

/// #998 and #1020 item 2 (REQ-change-relay-repair-001): a damaged relay in an approval a
/// correction embeds, with no committed copy, is removed; the correction, its other approval and
/// that approval's clean relay stay.
#[test]
fn repair_removes_a_damaged_correction_relay_with_no_committed_copy() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap();
    let path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let closing_actor = value["corrections"][0]["superseded_closing_approval"]["actor"]
        .as_str()
        .unwrap()
        .to_string();
    value["corrections"][0]["superseded_definition_approval"]["relayed_by"] =
        serde_json::json!({"decided_by": "Mallory", "words": "w", "where": "x", "recorded_by": "a"});
    let clean = serde_json::json!({
        "decided_by": closing_actor, "words": "w", "where": "x", "recorded_by": "a"
    });
    value["corrections"][0]["superseded_closing_approval"]["relayed_by"] = clean.clone();
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    let report = repair_relayed_decisions(root, &record.id).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["repairs"].as_array().map(Vec::len), Some(1), "{json:#}");
    let repair = &json["repairs"][0];
    assert_eq!(repair["file"], CORRECTIONS_FILE, "{json:#}");
    assert_eq!(repair["record"], 1, "{json:#}");
    assert_eq!(repair["within"], "superseded_definition_approval", "{json:#}");
    assert_eq!(repair["action"], "removed", "{json:#}");
    assert_eq!(repair["removed"]["decided_by"], "Mallory", "{json:#}");

    let after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let corrections = after["corrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 1);
    assert!(corrections[0]["superseded_definition_approval"]
        .get("relayed_by")
        .is_none());
    assert_eq!(corrections[0]["superseded_closing_approval"]["relayed_by"], clean);
}

/// #998 (REQ-cmd-change-relay-repair-001): the printed place names a record nested in a
/// reopening or a correction.
#[test]
fn a_nested_repair_names_its_place() {
    let repair = |file: &str, within: Option<&str>| RelayRepair {
        file: file.into(),
        record: 2,
        within: within.map(String::from),
        action: "removed".into(),
        from: None,
        removed: None,
    };
    assert_eq!(
        repair("approvals.json", None).place(),
        "approvals.json approval 2"
    );
    assert_eq!(
        repair("approvals.json", Some("superseded_approval")).place(),
        "approvals.json reopening 2 superseded approval"
    );
    assert_eq!(
        repair(CORRECTIONS_FILE, Some("superseded_definition_approval")).place(),
        "corrections.json correction 2 definition approval"
    );
    assert_eq!(
        repair(CORRECTIONS_FILE, Some("superseded_closing_approval")).place(),
        "corrections.json correction 2 closing approval"
    );
    assert_eq!(
        repair(SCOPED_REVIEW_ATTEMPTS_FILE, None).place(),
        "review-attempts.json attempt 2"
    );
    assert_eq!(repair(SCOPED_REVIEW_FILE, None).place(), "review.json");
}

/// #973 (REQ-change-relayed-decision-001, REQ-change-relay-repair-001): a malformed relay in an
/// approval a `corrections.json` correction embeds, or one naming someone other than that
/// approval's actor, is refused with the recovery a damaged relay anywhere else gets,
/// `repair-relays` for an active change, and that repair then fixes it.
#[test]
fn a_malformed_correction_relay_is_refused_with_its_recovery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let record = accept_completed_record(root, completed_no_spec_record(root));
    correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap();
    let record = load_change(root, &record.id).unwrap();
    let path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    value["corrections"][0]["superseded_definition_approval"]["relayed_by"] =
        serde_json::Value::Null;
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    let error = load_correction_ledger(root, &record).unwrap_err();
    assert!(error.contains("relayed_by"), "{error}");
    assert!(
        error.contains(&format!("specsync change repair-relays -- '{}'", record.id)),
        "{error}"
    );

    let report = repair_relayed_decisions(root, &record.id).unwrap();
    assert_eq!(report.repairs.len(), 1, "{report:?}");
    load_correction_ledger(root, &record).unwrap();

    // A well-formed relay naming someone other than the approval's actor gets the same recovery.
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    value["corrections"][0]["superseded_definition_approval"]["relayed_by"] =
        serde_json::json!({"decided_by": "Mallory", "words": "w", "where": "x", "recorded_by": "a"});
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    let error = load_correction_ledger(root, &record).unwrap_err();
    assert!(error.contains("says `Mallory` decided"), "{error}");
    assert!(
        error.contains(&format!("specsync change repair-relays -- '{}'", record.id)),
        "{error}"
    );
    repair_relayed_decisions(root, &record.id).unwrap();
    load_correction_ledger(root, &record).unwrap();
}
