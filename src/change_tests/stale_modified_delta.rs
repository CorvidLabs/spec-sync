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
