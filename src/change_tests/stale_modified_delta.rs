// Issue #843: a stale `## MODIFIED` delta must never silently overwrite newer canonical text.
//
// A `## MODIFIED` body is a full replacement written against the canonical block as it stood when
// the change was approved. When the base branch later changes that same block (another change
// lands and is merged in), re-materializing the stale body deletes the other change's text, and
// before this fix `change check` exited 0 and `change audit` passed over the loss.
//
// The fixtures below reproduce that order exactly: approve, (optionally) materialize, then the
// canonical block moves under the change the way a merge of main moves it, then check again.

const STALE_BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n";

/// The text another change appended to REQ-auth-001 on main after this change was approved.
const OTHER_CHANGE_PARAGRAPH: &str = "The system SHALL refuse a session reply that names no change.";

const STALE_MODIFIED_DELTA: &str = "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep one session per user and record each transition.\n\nAcceptance Criteria\n- One session per user.\n- Each transition is recorded.\n";

fn requirements_path(root: &Path) -> PathBuf {
    root.join("specs/auth/requirements.md")
}

/// Main's version of REQ-auth-001 after the other change landed: the old block plus a paragraph
/// and an acceptance criterion this change never saw.
fn main_moved_requirements() -> String {
    format!(
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\n{OTHER_CHANGE_PARAGRAPH}\n\nAcceptance Criteria\n- One session per user.\n- A reply naming no change is refused.\n"
    )
}

fn change_approved_against_old_requirement(root: &Path) -> ChangeRecord {
    let record = completed_section_only_current_record(root, STALE_MODIFIED_DELTA);
    fs::write(requirements_path(root), STALE_BASE_REQUIREMENTS).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap()
}

fn assert_refusal_names_the_block(error: &str, record: &ChangeRecord) {
    assert!(
        error.contains("REQ-auth-001"),
        "the refusal must name the requirement whose canonical text moved: {error}"
    );
    assert!(
        error.contains("`auth`"),
        "the refusal must name the module: {error}"
    );
    assert!(
        error.contains(&format!("specsync change approve {}", record.id)),
        "the refusal must name the re-approval that records the rebased body: {error}"
    );
}

/// DISCRIMINATOR for #843, the exact order chough#77 hit: the change has already materialized,
/// then main changes the same requirement and is merged in. On the unfixed binary the second
/// `materialize_change_deltas` returns `Ok` and writes the stale body over the other change's
/// paragraph.
#[test]
fn a_stale_modified_delta_never_overwrites_text_main_added_after_materialization() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    materialize_change_deltas(root, &record.id).unwrap();
    assert!(
        fs::read_to_string(requirements_path(root))
            .unwrap()
            .contains("record each transition"),
        "the fixture must materialize the approved body first"
    );

    // The merge of main: the other change's REQ-auth-001 wins the conflict.
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert_refusal_names_the_block(&error, &record);
    assert_eq!(
        fs::read_to_string(requirements_path(root)).unwrap(),
        merged,
        "a refused check must leave the canonical requirement byte for byte as main left it"
    );
}

/// DISCRIMINATOR for #843 before the first check: main moves the requirement between approval
/// and the first materialization. The stale body must not replace text nobody approved removing.
#[test]
fn a_stale_modified_delta_never_overwrites_text_main_added_before_the_first_check() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert_refusal_names_the_block(&error, &record);
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), merged);
}

/// DISCRIMINATOR for the audit half of #843: the packet's `change audit` passed over the stale
/// delta. Audit must report the same refusal `change check` gives, without writing anything.
#[test]
fn audit_reports_a_stale_modified_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();

    let error = ensure_approved_delta_bases_current(root, &load_change(root, &record.id).unwrap())
        .unwrap_err();

    assert_refusal_names_the_block(&error, &record);
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), merged);
}

/// The way forward the refusal names: rebase the delta body onto main's text and re-approve.
/// The rebased body then materializes, and the other change's paragraph survives because the
/// rebased body carries it.
#[test]
fn a_rebased_and_re_approved_delta_materializes_over_the_moved_requirement() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    materialize_change_deltas(root, &record.id).unwrap();
    fs::write(requirements_path(root), main_moved_requirements()).unwrap();
    materialize_change_deltas(root, &record.id).unwrap_err();

    let rebased = format!(
        "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep one session per user and record each transition.\n\n{OTHER_CHANGE_PARAGRAPH}\n\nAcceptance Criteria\n- One session per user.\n- A reply naming no change is refused.\n- Each transition is recorded.\n"
    );
    fs::write(delta_path(root, &record, "auth"), &rebased).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();

    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    assert!(requirements.contains(OTHER_CHANGE_PARAGRAPH), "{requirements}");
    assert!(requirements.contains("record each transition"), "{requirements}");
}

/// CONTROL: main moving a DIFFERENT block is not a conflict. The change's own block is still
/// applied, so check converges without refusing and without touching main's text.
#[test]
fn main_moving_an_unrelated_block_does_not_refuse_the_check() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let mut requirements = fs::read_to_string(requirements_path(root)).unwrap();
    requirements.push_str(
        "\n### REQ-auth-002\n\nThe system SHALL log out idle users.\n\nAcceptance Criteria\n- Idle users are logged out.\n",
    );
    fs::write(requirements_path(root), &requirements).unwrap();

    materialize_change_deltas(root, &record.id).unwrap();

    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements);
}

/// CONTROL: an approval written before this binding existed records no base, which is unknown,
/// not violated (the same reading REQ-change-089 gives absent delta digests). Such a change keeps
/// today's behaviour rather than failing on evidence nobody could have written.
#[test]
fn an_approval_without_recorded_bases_keeps_materializing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    let mut ledger = load_approvals(root, &record).unwrap();
    for approval in &mut ledger.approvals {
        approval.approved_delta_bases = None;
    }
    write_json(&change_dir(root, &record.id).join("approvals.json"), &ledger).unwrap();

    materialize_change_deltas(root, &record.id).unwrap();

    assert!(
        fs::read_to_string(requirements_path(root))
            .unwrap()
            .contains("record each transition")
    );
}

// ---------------------------------------------------------------------------------------------
// Review follow-ups on #845: acceptance, a withdrawn base binding, re-approval of an unchanged
// stale body, the real audit entry point, REMOVED, SPEC SECTION and more than one module.
// ---------------------------------------------------------------------------------------------

fn approved_change_with_delta(root: &Path, delta: &str) -> ChangeRecord {
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), STALE_BASE_REQUIREMENTS).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap()
}

fn spec_path(root: &Path) -> PathBuf {
    root.join("specs/auth/auth.spec.md")
}

/// DISCRIMINATOR for acceptance (#845 review, blocking 1). Acceptance is the second place deltas
/// reach the canonical spec, so a block main rewrote after the change reached verifying must be
/// refused there too, before anything is written.
#[test]
fn acceptance_refuses_a_stale_modified_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, STALE_MODIFIED_DELTA);
    let testing = change_dir(root, &record.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap_or_default();
    content.push_str(
        "\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-001 | transition record test |\n",
    );
    fs::write(&testing, content).unwrap();
    fs::write(requirements_path(root), STALE_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let record = start_implementation(root, &record.id).unwrap();
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed, "{verification:?}");
    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Verifying);
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();
    // Verification that is current for the moved tree, as acceptance sees it when the deltas
    // reach the canonical spec there rather than in `check` (for example a verification another
    // binary recorded): only the base guard stands between the stale body and main's text.
    let mut verification = load_verification(root, &record).unwrap();
    verification.workspace_digest = project_input_digest(root).unwrap();
    write_json(&change_dir(root, &record.id).join("verification.json"), &verification).unwrap();
    let ledger_before = fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap();

    let error = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap_err();

    assert_refusal_names_the_block(&error, &record);
    assert_eq!(
        fs::read_to_string(requirements_path(root)).unwrap(),
        merged,
        "a refused acceptance must leave main's requirement byte for byte"
    );
    assert_eq!(
        fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap(),
        ledger_before,
        "a refused acceptance must not record anything"
    );
    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Verifying);
}

/// DISCRIMINATOR for a withdrawn base binding (#845 review, blocking 2). Once a definition
/// approval on this change recorded canonical bases, a later definition approval that records
/// none is a claim being withdrawn, not silence from before the binding, and it must not turn
/// the guard off. Refused whether or not the block has moved, like the delta-digest binding.
#[test]
fn a_later_approval_that_drops_recorded_bases_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    let last = ledger
        .approvals
        .iter()
        .rposition(|approval| approval.gate == "definition")
        .unwrap();
    assert!(ledger.approvals[..last]
        .iter()
        .any(|approval| approval.gate == "definition" && approval.approved_delta_bases.is_some()));
    ledger.approvals[last].approved_delta_bases = None;
    write_json(&change_dir(root, &record.id).join("approvals.json"), &ledger).unwrap();
    let before = fs::read_to_string(requirements_path(root)).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        error.contains("records no canonical bases"),
        "a withdrawn base binding must be refused by name: {error}"
    );
    assert!(error.contains(&format!("specsync change approve {}", record.id)), "{error}");
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), before);

    // And with the block moved, the stale body still never lands.
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();
    materialize_change_deltas(root, &record.id).unwrap_err();
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), merged);
    let report = audit_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|entry| entry.contains("records no canonical bases")),
        "audit must report the withdrawn binding: {:?}",
        report.errors
    );
}

/// DISCRIMINATOR for re-approval (#845 review, non-blocking 1). Re-approving without rewriting
/// the body would record main's newer text as the base and let the stale body overwrite it. The
/// approval must be refused, naming the block, and nothing may be recorded.
#[test]
fn re_approving_an_unchanged_stale_body_is_refused() {
    for materialize_first in [true, false] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = change_approved_against_old_requirement(root);
        if materialize_first {
            materialize_change_deltas(root, &record.id).unwrap();
        }
        let merged = main_moved_requirements();
        fs::write(requirements_path(root), &merged).unwrap();
        let ledger_path = change_dir(root, &record.id).join("approvals.json");
        let ledger_before = fs::read_to_string(&ledger_path).unwrap();

        let error = approve_definition(root, &record.id, Some("Scope owner".into()), None)
            .unwrap_err();

        assert!(error.contains("REQ-auth-001"), "{materialize_first}: {error}");
        assert!(error.contains("`auth`"), "{materialize_first}: {error}");
        assert!(
            error.contains("rewrite each listed delta body onto the current canonical text"),
            "{materialize_first}: the refusal must name rewriting the body first: {error}"
        );
        assert_eq!(fs::read_to_string(&ledger_path).unwrap(), ledger_before);
        materialize_change_deltas(root, &record.id).unwrap_err();
        assert_eq!(
            fs::read_to_string(requirements_path(root)).unwrap(),
            merged,
            "{materialize_first}: the stale body must never land"
        );
    }
}

/// CONTROL for re-approval: re-approving an unchanged body whose block has NOT moved still works,
/// including after materialization, and keeps protecting the block afterwards.
#[test]
fn re_approving_an_unchanged_current_body_keeps_its_base() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    let first_bases = load_approvals(root, &record).unwrap().approvals.last().unwrap().approved_delta_bases.clone();
    materialize_change_deltas(root, &record.id).unwrap();

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    let ledger = load_approvals(root, &record).unwrap();
    assert_eq!(
        ledger.approvals.last().unwrap().approved_delta_bases,
        first_bases,
        "an applied, unchanged body carries the base it was approved against"
    );
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert_refusal_names_the_block(&error, &record);
}

/// COVERAGE (passes on 3007ab1 too) for the audit half, through `audit_project` itself (#845 review): the project
/// health check reports exactly the refusal `change check` gives.
#[test]
fn audit_project_reports_a_stale_modified_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let clean = audit_project(root);
    assert!(
        !clean
            .errors
            .iter()
            .any(|error| error.contains("approved against canonical text that has changed")),
        "the fixture must start clean: {:?}",
        clean.errors
    );
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();
    let refusal = materialize_change_deltas(root, &record.id).unwrap_err();

    let report = audit_project(root);

    assert!(
        report.errors.contains(&format!("{}: {refusal}", record.id)),
        "audit must report the refusal `change check` gives: {:?}",
        report.errors
    );
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), merged);
}

/// COVERAGE (passes on 3007ab1 too) for `## REMOVED`: main rewrote the requirement this change removes after the
/// change was approved. Removing it would delete text nobody approved removing.
#[test]
fn a_stale_removed_delta_never_deletes_text_main_added() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record =
        approved_change_with_delta(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    let merged = main_moved_requirements();
    fs::write(requirements_path(root), &merged).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert_refusal_names_the_block(&error, &record);
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), merged);
}

/// CONTROL for `## REMOVED`: an unchanged block is removed, and once removed it stays converged.
#[test]
fn a_current_removed_delta_removes_its_block() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record =
        approved_change_with_delta(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    materialize_change_deltas(root, &record.id).unwrap();
    assert!(!fs::read_to_string(requirements_path(root)).unwrap().contains("### REQ-auth-001"));
    materialize_change_deltas(root, &record.id).unwrap();
}

/// COVERAGE (passes on 3007ab1 too) for `## MODIFIED` of a SPEC SECTION: the refusal names the section.
#[test]
fn a_stale_spec_section_delta_never_overwrites_text_main_added() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_change_with_delta(
        root,
        "## MODIFIED\n\n### SPEC SECTION Legacy Notes\n\nRetained for compatibility and documented.\n",
    );
    let spec = fs::read_to_string(spec_path(root)).unwrap();
    let merged = spec.replace(
        "Retained for compatibility.\n",
        "Retained for compatibility.\n\nAnother change documented the migration path.\n",
    );
    assert_ne!(spec, merged);
    fs::write(spec_path(root), &merged).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("section `Legacy Notes`"), "{error}");
    assert!(error.contains("`auth`"), "{error}");
    assert!(error.contains(&format!("specsync change approve {}", record.id)), "{error}");
    assert_eq!(fs::read_to_string(spec_path(root)).unwrap(), merged);
}

/// COVERAGE (passes on 3007ab1 too) across modules: only the module whose block moved is named, and the refusal
/// comes before anything is written, so the module that did not move is not half-applied.
#[test]
fn a_stale_delta_in_one_module_is_named_and_blocks_every_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_section_only_current_record(root, STALE_MODIFIED_DELTA);
    fs::write(requirements_path(root), STALE_BASE_REQUIREMENTS).unwrap();
    fs::create_dir_all(root.join("specs/billing")).unwrap();
    fs::write(root.join("src/billing.rs"), "// Billing module.\n").unwrap();
    let billing_spec = fs::read_to_string(spec_path(root))
        .unwrap()
        .replace("module: auth", "module: billing")
        .replace("src/auth.rs", "src/billing.rs")
        .replace("# Auth\n", "# Billing\n")
        .replace("\nAuth.\n", "\nBilling.\n");
    fs::write(root.join("specs/billing/billing.spec.md"), &billing_spec).unwrap();
    let billing_requirements = "---\nspec: billing.spec.md\n---\n\n# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once per month.\n\nAcceptance Criteria\n- One bill per month.\n";
    let billing_requirements_path = root.join("specs/billing/requirements.md");
    fs::write(&billing_requirements_path, billing_requirements).unwrap();
    record.affected_specs = vec!["auth".into(), "billing".into()];
    save_change(root, &record).unwrap();
    fs::write(
        delta_path(root, &record, "billing"),
        "## MODIFIED\n\n### REQUIREMENT REQ-billing-001\n\nThe system SHALL bill once per month and itemize it.\n\nAcceptance Criteria\n- One bill per month.\n- Each bill is itemized.\n",
    )
    .unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let bases = load_approvals(root, &record)
        .unwrap()
        .approvals
        .last()
        .unwrap()
        .approved_delta_bases
        .clone()
        .expect("a new approval records bases");
    assert!(bases.contains_key("auth: REQUIREMENT REQ-auth-001"), "{bases:?}");
    assert!(bases.contains_key("billing: REQUIREMENT REQ-billing-001"), "{bases:?}");

    let moved_billing = billing_requirements.replace(
        "The system SHALL bill once per month.\n",
        "The system SHALL bill once per month.\n\nThe system SHALL send a receipt.\n",
    );
    fs::write(&billing_requirements_path, &moved_billing).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("billing: REQ-billing-001"), "{error}");
    assert!(!error.contains("REQ-auth-001"), "only the moved block is named: {error}");
    assert_eq!(fs::read_to_string(&billing_requirements_path).unwrap(), moved_billing);
    assert_eq!(
        fs::read_to_string(requirements_path(root)).unwrap(),
        STALE_BASE_REQUIREMENTS,
        "the refusal comes before any module is written"
    );
}

/// DISCRIMINATOR for a `## MODIFIED` `Change Log` section (#845 review plan, 2.2f). The change's
/// own materialization appends its row to that section, so the block never equals the approved
/// body afterwards and a recorded base would refuse the change's second check on its own text.
/// The Change Log therefore records no base, and a re-check is not refused as stale.
#[test]
fn a_modified_change_log_section_is_not_refused_as_stale_by_its_own_row() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_change_with_delta(
        root,
        "## MODIFIED\n\n### SPEC SECTION Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial (restated) |\n",
    );
    let bases = load_approvals(root, &record)
        .unwrap()
        .approvals
        .last()
        .unwrap()
        .approved_delta_bases
        .clone()
        .expect("a new approval records bases");
    assert!(
        !bases.keys().any(|key| key.ends_with("SPEC SECTION Change Log")),
        "the Change Log section must record no base: {bases:?}"
    );
    materialize_change_deltas(root, &record.id).unwrap();

    let second = materialize_change_deltas(root, &record.id);

    assert!(
        second
            .as_ref()
            .err()
            .is_none_or(|error| !error.contains("approved against canonical text that has changed")),
        "the change's own Change Log row must not read as a stale base: {second:?}"
    );
}

/// DISCRIMINATOR (review nit on #845): a canonical file that exists but cannot be read is an
/// error to surface, not an absent block. Reading it as empty would misreport an IO failure as
/// main having moved the requirement.
#[test]
fn an_unreadable_canonical_file_is_reported_not_treated_as_a_stale_base() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_approved_against_old_requirement(root);
    fs::remove_file(requirements_path(root)).unwrap();
    fs::create_dir_all(requirements_path(root)).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("could not read") && error.contains("requirements.md"),
        "the refusal must name the canonical file it could not read: {error}"
    );
    assert!(
        !error.contains("has changed since"),
        "an IO failure must not be reported as a stale base: {error}"
    );
}
