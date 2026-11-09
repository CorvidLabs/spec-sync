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
    let approvals_path = format!("{directory}/{APPROVALS_FILE}");
    let verification_path = format!("{directory}/{VERIFICATION_FILE}");
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
