// Issue #892 (audit SS-M05): what the open-task gate counts and how its one sentence reads. The
// gate itself runs through the binary in `open_task_gate_refuses_and_names_the_command_that_recovers`.

/// Verifies REQ-change-131: the line is the file's own 1-based line, ticked tasks and fenced
/// examples are not open, and the text is what follows the box.
#[test]
fn open_tasks_name_their_line_and_text_and_skip_fenced_examples() {
    let content = b"---\nchange: x\nartifact: tasks\n---\n\n# Tasks\n\n- [x] Done already.\n- [ ] Prove the fix.\n\n```markdown\n- [ ] An example, not a task.\n```\n\n* [ ]\n  - [X] Nested and done.\n+ [ ]   Spaced out.  \n";
    assert_eq!(
        open_tasks_in(content),
        vec![
            OpenTask {
                line: 9,
                text: "Prove the fix.".into(),
            },
            OpenTask {
                line: 15,
                text: String::new(),
            },
            OpenTask {
                line: 17,
                text: "Spaced out.".into(),
            },
        ]
    );
}

/// Verifies REQ-change-131: one task reads in the singular and names its file, line and text.
#[test]
fn one_open_task_reads_as_one_sentence() {
    let open = open_tasks_in(b"# Tasks\n\n- [ ] Prove the fix.\n");
    assert_eq!(
        open_tasks_sentence("guide", &open),
        "finish the 1 open task in `.specsync/changes/guide/tasks.md` (line 3: \"Prove the fix.\") and mark it `[x]`"
    );
}

/// Verifies REQ-change-131: at most five tasks are named, and the rest are counted, not dropped.
#[test]
fn open_tasks_beyond_five_are_counted() {
    let content: String = (1..=7).map(|index| format!("- [ ] Task {index}.\n")).collect();
    let open = open_tasks_in(content.as_bytes());
    let sentence = open_tasks_sentence("guide", &open);
    assert!(sentence.starts_with("finish the 7 open tasks in "), "{sentence}");
    for named in 1..=5 {
        assert!(sentence.contains(&format!("line {named}: \"Task {named}.\"")), "{sentence}");
    }
    assert!(!sentence.contains("Task 6."), "{sentence}");
    assert!(sentence.contains("; and 2 more) and mark each one `[x]`"), "{sentence}");
}

/// Task text is user content quoted into a terminal line: no control characters, no quote marks
/// or backticks that would end the quote or a command, and a bounded length.
#[test]
fn quoted_task_text_is_plain_and_bounded() {
    assert_eq!(
        quotable_task_text(" Run `cargo test` and say \"done\"\u{1b}[31m "),
        "Run 'cargo test' and say 'done'[31m"
    );
    let long = "x".repeat(OPEN_TASK_TEXT_CHARS + 20);
    let quoted = quotable_task_text(&long);
    assert_eq!(quoted.chars().count(), OPEN_TASK_TEXT_CHARS + 3);
    assert!(quoted.ends_with("..."));
    assert_eq!(quotable_task_text(&"y".repeat(OPEN_TASK_TEXT_CHARS)).len(), OPEN_TASK_TEXT_CHARS);
}

/// The refusal keeps the text it had before #892 as its start, so anything matching that still
/// matches, and names a check invocation's flags in the command it asks to re-run.
#[test]
fn a_check_refusal_names_its_own_flags() {
    assert_eq!(
        CheckInvocation::default().command("guide"),
        "specsync change check guide"
    );
    assert_eq!(
        CheckInvocation {
            strict: true,
            commit: true,
            push: true,
        }
        .command("guide"),
        "specsync change check guide --strict --commit --push"
    );
    assert_eq!(OPEN_TASKS_REFUSAL, "tasks.md contains incomplete tasks");
}

/// A reviewer claim with a space is one shell word in the command a review refusal names.
#[test]
fn a_reviewer_with_a_space_is_quoted_once() {
    assert_eq!(shell_quoted_reviewer("Fixture Reviewer"), "'Fixture Reviewer'");
    assert_eq!(shell_quoted_reviewer("0xLeif"), "0xLeif");
}
