use super::super::render::{
    INVALID_CORRECTION_LEDGER_TEXT, ensure_text_correction_ledger_valid, print_mutation_record,
    text_mode_next_action,
};
use crate::change;
use crate::change::ChangeKind;
use crate::change::CreateChangeRequest;
use crate::types::OutputFormat;
use std::fs;
use tempfile::TempDir;

#[test]
fn draft_text_surfaces_require_complete_artifacts_before_approval() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let mut record = change::create_change(
        root,
        CreateChangeRequest {
            description: "Clarify contributor guidance".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/commands/change.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some(
                "Documentation-only behavior does not alter a technical contract".into(),
            ),
        },
    )
    .expect("create draft");
    for (question, answer) in [
        (
            "acceptance_criteria",
            "Every draft text surface requires complete artifacts before approval",
        ),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        record = change::answer_question(root, &record.id, question, answer)
            .expect("answer interview question");
    }
    let questions = change::next_questions_for(root, &record);
    assert!(questions.is_empty(), "interview must be complete");

    for surface in ["status", "show", "list"] {
        let next = text_mode_next_action(root, &record, &questions);
        assert!(
            next.contains("complete selected artifacts"),
            "{surface} text recommended the wrong next action: {next}"
        );
        assert!(
            !next.contains("change approve"),
            "{surface} text recommended premature approval: {next}"
        );
    }
}
// Verifies REQ-cmd-change-009.
#[test]
fn text_correction_ledger_gate_uses_a_safe_diagnostic() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    change::write_default_policy(root, Vec::new()).expect("write default policy");
    let record = change::create_change(
        root,
        CreateChangeRequest {
            description: "Protect text lifecycle views".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Command test fixture".into()),
        },
    )
    .expect("create draft");
    let ledger_path = root
        .join(".specsync/changes")
        .join(&record.id)
        .join("corrections.json");
    fs::write(&ledger_path, "{ malformed correction ledger\\n").expect("write malformed ledger");

    let error = ensure_text_correction_ledger_valid(root, &record).expect_err("invalid ledger");

    assert_eq!(error, INVALID_CORRECTION_LEDGER_TEXT);
    assert!(!error.contains("malformed correction ledger"));
}

// Verifies REQ-cmd-change-010.
#[test]
fn mutation_output_uses_the_in_transaction_correction_snapshot() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    change::write_default_policy(root, Vec::new()).expect("write default policy");
    let record = change::create_change(
        root,
        CreateChangeRequest {
            description: "Render one successful lifecycle mutation".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Command test fixture".into()),
        },
    )
    .expect("create draft");
    let result = change::answer_question_with_snapshot(
        root,
        &record.id,
        "acceptance_criteria",
        "The successful mutation renders from its validated transaction snapshot",
    )
    .expect("persist mutation");
    let ledger_path = root
        .join(".specsync/changes")
        .join(&record.id)
        .join("corrections.json");
    fs::write(&ledger_path, "{ malformed correction ledger\\n")
        .expect("corrupt ledger after persistence");

    assert!(change::effective_change_definition(root, &result.change).is_err());
    assert!(!change::summarize_change(root, &result.change).correction_valid);
    assert!(result.summary.correction_valid);
    assert!(result.strict_summary.correction_valid);
    assert!(
        print_mutation_record(root, &record.id, &result, OutputFormat::Text, true, false).is_ok()
    );
    assert!(
        print_mutation_record(root, &record.id, &result, OutputFormat::Json, true, false).is_ok()
    );
}
