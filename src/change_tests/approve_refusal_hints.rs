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

/// Verifies REQ-change-133 (#958). DISCRIMINATOR: a question with one choice names that choice
/// itself, so the command runs as printed; `<none>` is a Bash input redirection.
#[test]
fn a_single_choice_question_names_its_choice_literally() {
    let question = |id: &str, choices: &[&str]| InterviewQuestion {
        id: id.into(),
        prompt: "prompt".into(),
        choices: choices.iter().map(|choice| choice.to_string()).collect(),
        recommended: choices.first().map(|choice| choice.to_string()),
    };
    let single = interview_incomplete_refusal("add-auth", &[question("constraints", &["none"])]);
    assert!(
        single.contains("run `specsync change answer add-auth constraints none`"),
        "a single choice is the answer itself, so the command runs as printed: {single}"
    );
    assert!(
        !single.contains("<none>"),
        "`<none>` reads as an input redirection in Bash: {single}"
    );
    assert!(
        single.ends_with(", or name the lessons or principles that constrain it instead"),
        "DOCS-3.b: `constraints` is free text with a `none` shortcut, so the refusal still says \
         the lessons that apply can be named instead: {single}"
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
    let free = interview_incomplete_refusal("add-auth", &[question("summary", &[])]);
    assert!(
        free.ends_with("run `specsync change answer add-auth summary \"<answer>\"`"),
        "a question without choices takes free text: {free}"
    );
    // The scope questions keep the placeholders approve's scope refusals give, so status and
    // approve name one command for each (DOCS-3.c).
    for (scope, placeholder) in [
        ("acceptance_criteria", "\"<observable outcome>\""),
        ("affected_specs", "\"<module>\""),
        ("affected_paths", "\"<path>\""),
    ] {
        let named = interview_incomplete_refusal("add-auth", &[question(scope, &[])]);
        assert!(
            named.ends_with(&format!(
                "run `specsync change answer add-auth {scope} {placeholder}`"
            )),
            "{named}"
        );
    }
}

/// Verifies REQ-change-133 and DOCS-3.c (#958). DISCRIMINATOR: a draft missing both its specs
/// and its paths is refused for the question status asks first, `affected_specs`
/// (`next_questions` order), not `affected_paths`.
#[test]
fn a_draft_without_specs_or_paths_is_refused_for_the_question_status_asks_first() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut id = String::new();
    let mut first = String::new();
    let error = unfinished_definition_refusal(root, |record| {
        id = record.id.clone();
        record.no_spec_change = false;
        record.no_spec_change_rationale = None;
        record.affected_specs.clear();
        record.affected_paths.clear();
        first = next_questions(record)
            .first()
            .map(|question| question.id.clone())
            .unwrap_or_default();
    });
    assert_eq!(first, "affected_specs", "status asks for the specs first");
    assert!(
        error.starts_with("affected specs are required unless no_spec_change is justified")
            && error.contains(&format!(
                "run `specsync change answer {id} affected_specs \"<module>\"`"
            )),
        "approve must name the command status names for the same draft: {error}"
    );
}
