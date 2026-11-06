// Verifies REQ-change-133 (#917, from the Codex review of #912): a refusal for an unfinished
// definition names the `specsync change answer` command only where that command can run (a
// draft), covers the scope questions as well as the criteria and the interview, and shows a
// single choice as the placeholder.

/// A complete workflow-v2 draft, edited by `edit`, then validated.
fn unfinished_definition_refusal(root: &Path, edit: impl FnOnce(&mut ChangeRecord)) -> String {
    let mut record = completed_no_spec_current_record(root);
    edit(&mut record);
    save_change(root, &record).unwrap();
    validate_definition(root, &record).expect_err("the unfinished definition must be refused")
}

#[test]
fn a_draft_without_affected_paths_names_the_affected_paths_answer() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut id = String::new();
    let error = unfinished_definition_refusal(root, |record| {
        id = record.id.clone();
        record.affected_paths.clear();
    });
    assert!(
        error.starts_with("at least one affected path is required"),
        "the condition is unchanged: {error}"
    );
    assert!(
        error.contains(&format!(
            "run `specsync change answer {id} affected_paths \"<path>\"`"
        )),
        "a draft without paths must be told which answer finishes it: {error}"
    );
}

#[test]
fn a_draft_without_affected_specs_names_the_affected_specs_answer() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut id = String::new();
    let error = unfinished_definition_refusal(root, |record| {
        id = record.id.clone();
        record.no_spec_change = false;
        record.no_spec_change_rationale = None;
        record.affected_specs.clear();
    });
    assert!(
        error.starts_with("affected specs are required unless no_spec_change is justified"),
        "the condition is unchanged: {error}"
    );
    assert!(
        error.contains(&format!(
            "run `specsync change answer {id} affected_specs \"<module>\"`"
        )),
        "a draft without specs must be told which answer finishes it: {error}"
    );
}

#[test]
fn a_record_past_draft_is_never_told_to_run_change_answer() {
    // `change answer` refuses every state but draft, so naming it after approval would name a
    // command that cannot recover. These states reach the same validation through check,
    // verification and audit when a record has been corrupted or hand-edited.
    for state in [
        ChangeState::Approved,
        ChangeState::Implementing,
        ChangeState::Verifying,
    ] {
        for (expected, edit) in [
            (
                "at least one acceptance criterion is required",
                (|record: &mut ChangeRecord| record.acceptance_criteria.clear())
                    as fn(&mut ChangeRecord),
            ),
            ("at least one affected path is required", |record| {
                record.affected_paths.clear()
            }),
            ("the deterministic interview is incomplete", |record| {
                record.answers.remove("public_contract");
            }),
        ] {
            let temp = TempDir::new().unwrap();
            let root = temp.path();
            let error = unfinished_definition_refusal(root, |record| {
                record.state = state;
                edit(record);
            });
            assert!(
                error.starts_with(expected),
                "{state:?}: the condition is unchanged: {error}"
            );
            assert!(
                !error.contains("specsync change answer"),
                "{state:?}: `change answer` refuses outside draft, so it must not be named: {error}"
            );
        }
    }
}

#[test]
fn a_single_choice_question_shows_its_choice_as_the_placeholder() {
    let question = |id: &str, choices: &[&str]| InterviewQuestion {
        id: id.into(),
        prompt: "prompt".into(),
        choices: choices.iter().map(|choice| choice.to_string()).collect(),
        recommended: choices.first().map(|choice| choice.to_string()),
    };
    let single = interview_incomplete_refusal("add-auth", &[question("constraints", &["none"])]);
    assert!(
        single.ends_with("run `specsync change answer add-auth constraints <none>`"),
        "REQ-change-133 gives the question's choices as the placeholder: {single}"
    );
    let pair = interview_incomplete_refusal(
        "add-auth",
        &[
            question("public_contract", &["yes", "no"]),
            question("architecture_risk", &["yes", "no"]),
        ],
    );
    assert!(
        pair.contains("(still to answer: `public_contract`, `architecture_risk`)")
            && pair.ends_with("run `specsync change answer add-auth public_contract <yes|no>`"),
        "{pair}"
    );
    let free = interview_incomplete_refusal("add-auth", &[question("acceptance_criteria", &[])]);
    assert!(
        free.ends_with("run `specsync change answer add-auth acceptance_criteria \"<answer>\"`"),
        "a question without choices takes free text: {free}"
    );
}
