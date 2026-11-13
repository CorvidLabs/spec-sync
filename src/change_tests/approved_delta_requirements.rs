// REQ-change-113: the requirement IDs and spec sections an approved semantic delta declares, per
// module, with an integrity flag from the same binding read the summary uses and a
// materialization flag computed on the bound body.
//
// Labels follow `testing.md` of the change. A DISCRIMINATOR fails against unfixed a80748f with
// only a stub `approved_delta_summaries` (returning `[]`) and an empty
// `DefinitionMutationResult.approved_deltas`; a CONTROL passes there too.

/// The approved `auth` delta: an added requirement whose body mentions another ID in prose, a
/// modified section and a removed section.
const APPROVED_REQUIREMENT_DELTA_BODY: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL keep a session alive across a refresh. It replaces what REQ-auth-009 described.\n\nAcceptance Criteria\n- A session survives a page refresh.\n\n## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials and sessions.\n\n## REMOVED\n\n### SPEC SECTION Legacy Notes\n\nRetired.\n";

/// The same delta with the requirement wording changed after approval.
const EDITED_REQUIREMENT_DELTA_BODY: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL carry wording nobody approved.\n\nAcceptance Criteria\n- Nobody reviewed this.\n\n## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials and sessions.\n\n## REMOVED\n\n### SPEC SECTION Legacy Notes\n\nRetired.\n";

/// A workflow-v2 change whose `auth` delta above is approved, with requirement evidence for
/// REQ-auth-002 already in `testing.md`.
fn change_with_an_approved_requirement_delta(root: &Path) -> ChangeRecord {
    let record = completed_section_only_current_record(root, APPROVED_REQUIREMENT_DELTA_BODY);
    let testing = change_dir(root, &record.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap_or_default();
    content.push_str(
        "\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-002 | session refresh test |\n",
    );
    fs::write(&testing, content).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap()
}

fn declarations(added: &[&str], modified: &[&str], removed: &[&str]) -> DeltaDeclarations {
    let owned = |values: &[&str]| values.iter().map(|value| (*value).to_string()).collect();
    DeltaDeclarations {
        added: owned(added),
        modified: owned(modified),
        removed: owned(removed),
    }
}

fn approved_requirement_delta_entry(materialized: bool) -> ApprovedDeltaSummary {
    ApprovedDeltaSummary {
        module: "auth".into(),
        delta_intact: Some(true),
        requirements: Some(declarations(&["REQ-auth-002"], &[], &[])),
        sections: Some(declarations(&[], &["Purpose"], &["Legacy Notes"])),
        requirements_sections: None,
        materialized: Some(materialized),
        declarations_error: None,
    }
}

fn bare_entry(module: &str, delta_intact: Option<bool>) -> ApprovedDeltaSummary {
    ApprovedDeltaSummary {
        module: module.into(),
        delta_intact,
        requirements: None,
        sections: None,
        requirements_sections: None,
        materialized: None,
        declarations_error: None,
    }
}

/// Invariants 1 to 3 of the change's `design.md`, between one summary and one entry list.
fn assert_approved_delta_invariants(summary: &ChangeSummary, entries: &[ApprovedDeltaSummary]) {
    for entry in entries {
        let named = summary.changed_approved_deltas.contains(&entry.module);
        let refused_whole = !summary.approval_valid && summary.changed_approved_deltas.is_empty();
        assert_eq!(
            entry.delta_intact == Some(false),
            named || refused_whole,
            "invariant 1: not intact exactly when the summary names the module or refuses the \
             binding as a whole: {entry:?} {summary:?}"
        );
        let lists = entry.requirements.is_some()
            && entry.sections.is_some()
            && entry.materialized.is_some();
        let none = entry.requirements.is_none()
            && entry.sections.is_none()
            && entry.materialized.is_none();
        assert!(lists || none, "invariant 3: lists come together: {entry:?}");
        assert_eq!(
            lists,
            entry.delta_intact == Some(true) && entry.declarations_error.is_none(),
            "invariant 3: lists exactly for an intact, parsed body: {entry:?}"
        );
        if entry.declarations_error.is_some() {
            assert_eq!(entry.delta_intact, Some(true), "invariant 3: {entry:?}");
        }
    }
    if !entries.is_empty() {
        assert_eq!(
            summary.approval_valid,
            !entries
                .iter()
                .any(|entry| entry.delta_intact == Some(false)),
            "invariant 2: approval_valid exactly when no entry is not intact: {entries:?} {summary:?}"
        );
    }
}

fn assert_consistent_with_summary(root: &Path, record: &ChangeRecord) -> Vec<ApprovedDeltaSummary> {
    let _scope = begin_change_read_scope(root);
    let summary = summarize_change(root, record);
    let entries = approved_delta_summaries(root, record);
    assert_approved_delta_invariants(&summary, &entries);
    entries
}

/// Rewrite the effective definition approval's recorded digest for `module` to the digest of
/// `body`: the ledger an approval under a more permissive grammar would have left.
fn record_delta_digest_for(root: &Path, record: &ChangeRecord, module: &str, body: &str) {
    let mut ledger = load_approvals(root, record).unwrap();
    let approval = ledger
        .approvals
        .iter_mut()
        .rev()
        .find(|approval| approval.gate == "definition")
        .unwrap();
    approval
        .approved_delta_digests
        .as_mut()
        .unwrap()
        .insert(module.to_string(), delta_body_digest(module, body));
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
}

/// DISCRIMINATOR for REQ-change-113.
#[test]
fn approved_delta_summaries_report_declared_requirements_and_sections() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    assert_eq!(record.state, ChangeState::Approved);

    let entries = assert_consistent_with_summary(root, &record);

    assert_eq!(entries, vec![approved_requirement_delta_entry(false)]);
    let value = serde_json::to_value(&entries[0]).unwrap();
    assert!(value.get("declarations_error").is_none(), "{value}");
    assert_eq!(value["requirements"]["added"], serde_json::json!(["REQ-auth-002"]));
}

/// DISCRIMINATOR for REQ-change-113: only item headings are reported.
#[test]
fn a_requirement_mentioned_in_prose_is_not_reported() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    assert!(
        fs::read_to_string(delta_path(root, &record, "auth"))
            .unwrap()
            .contains("REQ-auth-009"),
        "the fixture must mention the prose ID in the approved body"
    );

    let entries = approved_delta_summaries(root, &record);

    let requirements = entries
        .first()
        .and_then(|entry| entry.requirements.clone())
        .expect("an intact approved delta reports its requirement declarations");
    assert_eq!(requirements.added, vec!["REQ-auth-002".to_string()]);
    for list in [&requirements.added, &requirements.modified, &requirements.removed] {
        assert!(!list.iter().any(|id| id == "REQ-auth-009"), "{requirements:?}");
    }
}

/// DISCRIMINATOR for REQ-change-113: an edited body reports no approved requirement.
#[test]
fn an_edited_approved_delta_reports_no_approved_requirements() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    fs::write(delta_path(root, &record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();

    let entries = assert_consistent_with_summary(root, &record);

    assert_eq!(entries, vec![bare_entry("auth", Some(false))]);
    let value = serde_json::to_value(&entries[0]).unwrap();
    for key in ["requirements", "sections", "materialized", "declarations_error"] {
        assert!(value.get(key).is_none(), "{key} must be withheld: {value}");
    }
    assert_eq!(value["delta_intact"], serde_json::json!(false));
    let summary = summarize_change(root, &record);
    assert_eq!(summary.changed_approved_deltas, vec!["auth".to_string()]);
    assert!(!summary.approval_valid);
}

/// DISCRIMINATOR for REQ-change-113: a withdrawn binding refuses every module.
#[test]
fn a_withdrawn_delta_binding_reports_every_module_not_intact() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    withdraw_the_recorded_delta_binding(root, &record);

    let entries = assert_consistent_with_summary(root, &record);

    assert_eq!(entries, vec![bare_entry("auth", Some(false))]);
    let summary = summarize_change(root, &record);
    assert!(!summary.approval_valid);
    assert!(summary.changed_approved_deltas.is_empty());
}

/// DISCRIMINATOR for REQ-change-113: a ledger that never recorded a digest is unknown.
#[test]
fn a_ledger_without_delta_digests_reports_unknown_integrity() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    strip_recorded_delta_digests(root, &record);

    let entries = assert_consistent_with_summary(root, &record);
    assert_eq!(entries, vec![bare_entry("auth", None)]);
    let value = serde_json::to_value(&entries[0]).unwrap();
    assert!(value["delta_intact"].is_null(), "{value}");
    assert!(value.get("requirements").is_none(), "{value}");

    fs::write(delta_path(root, &record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![bare_entry("auth", None)],
        "an unknown binding is never reported as violated"
    );
}

/// DISCRIMINATOR for REQ-change-113: an intact body the current grammar refuses says why.
#[test]
fn an_intact_delta_the_current_grammar_refuses_reports_a_declarations_error() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    let refused = "## RENAMED\n\n### SPEC SECTION Purpose\n\nAuth.\n";
    assert!(parse_delta(refused).is_err());
    fs::write(delta_path(root, &record, "auth"), refused).unwrap();
    record_delta_digest_for(root, &record, "auth", refused);

    let entries = assert_consistent_with_summary(root, &record);

    assert_eq!(entries.len(), 1, "{entries:?}");
    let entry = &entries[0];
    assert_eq!(entry.module, "auth");
    assert_eq!(entry.delta_intact, Some(true));
    assert!(
        entry
            .declarations_error
            .as_deref()
            .is_some_and(|error| error.contains("## RENAMED")),
        "{entry:?}"
    );
    assert!(entry.requirements.is_none() && entry.sections.is_none());
    assert!(entry.materialized.is_none());
    let summary = summarize_change(root, &record);
    assert!(summary.approval_valid, "{summary:?}");
    assert!(summary.changed_approved_deltas.is_empty());
}

/// DISCRIMINATOR for REQ-change-113 and #741: `materialized` is derived from the tree.
#[test]
fn materialized_turns_true_after_check_and_false_after_a_reapproved_correction() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    let applied = materialize_change_deltas(root, &record.id).unwrap();
    assert!(applied.canonical_applied);
    assert_eq!(
        assert_consistent_with_summary(root, &applied),
        vec![approved_requirement_delta_entry(true)]
    );

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_materialized_then_corrected_and_re_approved(root);
    let record = load_change(root, &record.id).unwrap();
    assert!(record.canonical_applied);
    let corrected = ApprovedDeltaSummary {
        module: "auth".into(),
        delta_intact: Some(true),
        requirements: Some(declarations(&[], &[], &[])),
        sections: Some(declarations(&[], &["Purpose"], &[])),
        requirements_sections: None,
        materialized: Some(false),
        declarations_error: None,
    };
    assert_eq!(assert_consistent_with_summary(root, &record), vec![corrected.clone()]);

    let record = materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![ApprovedDeltaSummary {
            materialized: Some(true),
            ..corrected
        }]
    );
}

/// DISCRIMINATOR for REQ-change-113 in the implementing and verifying states.
#[test]
fn approved_delta_summaries_hold_in_implementing_and_verifying() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    let record = start_implementation(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Implementing);
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![approved_requirement_delta_entry(false)]
    );

    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed, "{verification:?}");
    let record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![approved_requirement_delta_entry(true)]
    );
}

/// DISCRIMINATOR for REQ-change-113 and #730: CRLF alone is not an edit.
#[test]
fn a_crlf_checkout_of_an_approved_delta_stays_intact() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);
    fs::write(
        delta_path(root, &record, "auth"),
        approved_delta_body_as_a_crlf_checkout_writes_it(),
    )
    .unwrap();

    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![ApprovedDeltaSummary {
            module: "auth".into(),
            delta_intact: Some(true),
            requirements: Some(declarations(&[], &[], &[])),
            sections: Some(declarations(&[], &["Purpose"], &[])),
            requirements_sections: None,
            materialized: Some(false),
            declarations_error: None,
        }]
    );
}

/// DISCRIMINATOR for REQ-change-113: restoring or re-approving returns the entry to intact.
#[test]
fn restoring_or_reapproving_an_edited_delta_restores_intact() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    let delta = delta_path(root, &record, "auth");
    fs::write(&delta, EDITED_REQUIREMENT_DELTA_BODY).unwrap();
    assert_eq!(
        approved_delta_summaries(root, &record),
        vec![bare_entry("auth", Some(false))]
    );

    fs::write(&delta, APPROVED_REQUIREMENT_DELTA_BODY).unwrap();
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![approved_requirement_delta_entry(false)]
    );

    let reworded = APPROVED_REQUIREMENT_DELTA_BODY.replace(
        "### REQUIREMENT REQ-auth-002",
        "### REQUIREMENT REQ-auth-003",
    );
    fs::write(&delta, &reworded).unwrap();
    let testing = change_dir(root, &record.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap();
    content.push_str("| REQ-auth-003 | reworded session test |\n");
    fs::write(&testing, content).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![ApprovedDeltaSummary {
            requirements: Some(declarations(&["REQ-auth-003"], &[], &[])),
            ..approved_requirement_delta_entry(false)
        }]
    );
}

/// DISCRIMINATOR for REQ-change-113: one read scope reads each delta once, so the summary and
/// `approved_deltas` cannot disagree about a delta edited between them.
#[test]
fn status_projections_share_one_delta_binding_read() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);

    let (summary, first, after_edit) = {
        let _scope = begin_change_read_scope(root);
        reset_test_delta_binding_read_count();
        let summary = summarize_change_with_strict(root, &record, false);
        let strict = summarize_change_with_strict(root, &record, true);
        let first = approved_delta_summaries(root, &record);
        assert_eq!(
            test_delta_binding_read_count(),
            record.affected_specs.len(),
            "one read scope reads each affected delta exactly once"
        );
        assert_eq!(summary.approval_valid, strict.approval_valid);
        fs::write(delta_path(root, &record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();
        let after_edit = approved_delta_summaries(root, &record);
        (summary, first, after_edit)
    };

    assert!(summary.approval_valid);
    assert_eq!(first, vec![approved_requirement_delta_entry(false)]);
    assert_eq!(
        after_edit, first,
        "within one scope the entries agree with the summary already built"
    );
    assert_approved_delta_invariants(&summary, &after_edit);
    assert_eq!(
        assert_consistent_with_summary(root, &record),
        vec![bare_entry("auth", Some(false))],
        "a fresh scope reads the edited body"
    );
}

/// DISCRIMINATOR for REQ-change-113 and REQ-cmd-change-023: `change depend` on an approved
/// change carries the entries, built under the lock with its summary.
///
/// Dependencies are part of the approved scope, so a NEW dependency makes the scope stale and
/// the entries `[]`, exactly as status then reports. Re-declaring a dependency the approval
/// already covers leaves the scope current, and the in-lock entries are the intact ones.
#[test]
fn a_dependency_added_to_an_approved_change_reports_its_approved_deltas() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let prerequisite = |root: &Path, base: &str| {
        create_change(
            root,
            CreateChangeRequest {
                description: distinct_fixture_description(root, base),
                kind: ChangeKind::Documentation,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Fixture prerequisite".into()),
            },
        )
        .unwrap()
    };
    let other = prerequisite(root, "an approved prerequisite");
    let draft = completed_section_only_current_record(root, APPROVED_REQUIREMENT_DELTA_BODY);
    let testing = change_dir(root, &draft.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap_or_default();
    content.push_str("\n| REQ-auth-002 | session refresh test |\n");
    fs::write(&testing, content).unwrap();
    add_dependency_with_snapshot(root, &draft.id, &other.id).unwrap();
    let record = approve_definition(root, &draft.id, Some("Scope owner".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Approved);

    let result = add_dependency_with_snapshot(root, &record.id, &other.id).unwrap();

    assert_eq!(result.change.state, ChangeState::Approved);
    assert!(result.summary.approval_valid, "{:?}", result.summary);
    assert_approved_delta_invariants(&result.summary, &result.approved_deltas);
    assert_eq!(
        result.approved_deltas,
        vec![approved_requirement_delta_entry(false)]
    );
    assert_eq!(
        result.approved_deltas,
        approved_delta_summaries(root, &result.change),
        "the in-lock entries are the entries status reports for the same record"
    );

    let unapproved = prerequisite(root, "an unapproved prerequisite");
    let widened = add_dependency_with_snapshot(root, &record.id, &unapproved.id).unwrap();
    assert!(!widened.summary.approval_valid, "a new dependency widens the approved scope");
    assert!(widened.approved_deltas.is_empty());
    assert_eq!(
        widened.approved_deltas,
        approved_delta_summaries(root, &widened.change)
    );
}

/// CONTROL: nothing is reported outside an active state with a current scope.
#[test]
fn approved_delta_summaries_are_empty_outside_active_approved_states() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let draft = completed_section_only_current_record(root, APPROVED_REQUIREMENT_DELTA_BODY);
    assert_eq!(draft.state, ChangeState::Draft);
    assert!(approved_delta_summaries(root, &draft).is_empty());

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut stale = change_with_an_approved_requirement_delta(root);
    stale.affected_paths.push("src/unrelated.rs".into());
    save_change(root, &stale).unwrap();
    write_change_markdown(root, &stale).unwrap();
    let stale = load_change(root, &stale.id).unwrap();
    assert!(ensure_definition_approval_valid(root, &stale).is_err());
    assert!(approved_delta_summaries(root, &stale).is_empty());

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let approved = change_with_an_approved_requirement_delta(root);
    for state in [ChangeState::Accepted, ChangeState::Archived] {
        let mut record = approved.clone();
        record.state = state;
        assert!(
            approved_delta_summaries(root, &record).is_empty(),
            "{state:?} reports nothing"
        );
    }

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let no_spec = completed_no_spec_current_record(root);
    let no_spec = approve_definition(root, &no_spec.id, Some("Scope owner".into()), None).unwrap();
    assert!(no_spec.no_spec_change);
    assert!(approved_delta_summaries(root, &no_spec).is_empty());
}

/// CONTROL: draft-only mutations carry an empty list.
#[test]
fn definition_mutations_on_drafts_report_no_approved_deltas() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let draft = completed_section_only_current_record(root, APPROVED_REQUIREMENT_DELTA_BODY);
    assert!(
        approved_delta_summaries(root, &draft).is_empty(),
        "`change new` renders the draft it created"
    );
    let answered =
        answer_question_with_snapshot(root, &draft.id, "public_contract", "no").unwrap();
    assert!(answered.approved_deltas.is_empty());
    let other = create_change(
        root,
        CreateChangeRequest {
            description: distinct_fixture_description(root, "a draft prerequisite"),
            kind: ChangeKind::Documentation,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Fixture prerequisite".into()),
        },
    )
    .unwrap();
    let depended = add_dependency_with_snapshot(root, &draft.id, &other.id).unwrap();
    assert_eq!(depended.change.state, ChangeState::Draft);
    assert!(depended.approved_deltas.is_empty());
}

/// CONTROL: the invariants hold on every fixture shape, including the `declarations_error` one.
#[test]
fn approved_delta_summaries_agree_with_changed_approved_deltas() {
    type Edit = fn(&Path, &ChangeRecord);
    let edits: [(&str, Edit); 6] = [
        ("intact", |_, _| {}),
        ("edited", |root, record| {
            fs::write(delta_path(root, record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();
        }),
        ("withdrawn", |root, record| {
            withdraw_the_recorded_delta_binding(root, record);
        }),
        ("unrecorded", |root, record| {
            strip_recorded_delta_digests(root, record);
        }),
        ("crlf", |root, record| {
            fs::write(
                delta_path(root, record, "auth"),
                APPROVED_REQUIREMENT_DELTA_BODY.replace('\n', "\r\n"),
            )
            .unwrap();
        }),
        ("grammar refused", |root, record| {
            let refused = "## RENAMED\n\n### SPEC SECTION Purpose\n\nAuth.\n";
            fs::write(delta_path(root, record, "auth"), refused).unwrap();
            record_delta_digest_for(root, record, "auth", refused);
        }),
    ];
    for (label, edit) in edits {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = change_with_an_approved_requirement_delta(root);
        edit(root, &record);
        let _scope = begin_change_read_scope(root);
        let summary = summarize_change(root, &record);
        let entries = approved_delta_summaries(root, &record);
        assert_approved_delta_invariants(&summary, &entries);
        assert!(
            entries.len() <= record.affected_specs.len(),
            "{label}: at most one entry per module: {entries:?}"
        );
    }
}

/// CONTROL: the refusing gates read the delta files live, outside any memo.
#[test]
fn gates_read_delta_bodies_live_outside_a_memo() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    assert!(ensure_approved_delta_bodies_unchanged(root, &record).is_ok());
    fs::write(delta_path(root, &record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();
    assert!(
        ensure_approved_delta_bodies_unchanged(root, &record)
            .unwrap_err()
            .contains("semantic delta for `auth` changed after approval")
    );
    assert!(materialize_change_deltas(root, &record.id).is_err());
}

/// CONTROL: the binding refactor keeps every existing output and refusal text.
#[test]
fn delta_binding_refactor_keeps_existing_refusals() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    assert_eq!(changed_approved_delta_modules(root, &record), Ok(Vec::new()));
    fs::write(delta_path(root, &record, "auth"), EDITED_REQUIREMENT_DELTA_BODY).unwrap();
    assert_eq!(
        changed_approved_delta_modules(root, &record),
        Ok(vec!["auth".to_string()])
    );
    assert_eq!(
        ensure_approved_delta_bodies_unchanged(root, &record).unwrap_err(),
        format!(
            "semantic delta for `auth` changed after approval; the approved wording is what rewrites the canonical spec, so re-run `specsync change approve {id}` to approve the current delta bodies and then `specsync change check {id}` to materialize them into the canonical spec (or restore the approved delta bodies)",
            id = record.id
        )
    );
    fs::write(delta_path(root, &record, "auth"), APPROVED_REQUIREMENT_DELTA_BODY).unwrap();
    withdraw_the_recorded_delta_binding(root, &record);
    assert_eq!(
        changed_approved_delta_modules(root, &record).unwrap_err(),
        format!(
            "the definition approval for {id} records no semantic delta wording although an earlier definition approval on this change recorded it; an approval cannot withdraw a claim an earlier one made, so re-run `specsync change approve {id}` to record the delta bodies this approval covers, then `specsync change check {id}` to bring the canonical spec to them (or restore the approval ledger)",
            id = record.id
        )
    );
}
