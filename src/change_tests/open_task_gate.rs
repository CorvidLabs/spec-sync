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

/// Codex review of #903: the review command a refusal names must parse for every claim the
/// validator accepts, including one that starts with `-` and one with a space.
#[test]
fn the_named_review_command_parses_for_every_valid_reviewer() {
    use clap::Parser;
    for reviewer in ["0xLeif", "-bot", "Fixture Reviewer", "ci:bot/@x-1"] {
        let claim = validate_scoped_reviewer_claim(reviewer).unwrap();
        let command = format!("specsync change review guide {}", reviewer_argument(claim));
        let words = shell_words(&command).unwrap();
        let parsed = crate::cli::Cli::try_parse_from(&words)
            .unwrap_or_else(|error| panic!("`{command}` does not parse: {error}"));
        match parsed.command {
            Some(crate::cli::Command::Change {
                action: crate::cli::ChangeAction::Review { reviewer: parsed, .. },
            }) => assert_eq!(parsed, reviewer, "{command}"),
            _ => panic!("`{command}` is not a review"),
        }
    }
}

/// Verifies REQ-change-131 (Codex review of #903). DISCRIMINATOR: a finalization that stopped
/// after acceptance is retried with its workspace still active and a task reopened in between.
/// The retry refuses and archives nothing; ticking the box and running the named command archives.
#[test]
fn a_resumed_finalization_still_in_its_workspace_refuses_a_reopened_task() {
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
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);

    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    // Finalization stopped here: accepted, not yet moved into the archive.
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Accepted);

    let tasks = change_dir(root, &record.id).join("tasks.md");
    let done = fs::read_to_string(&tasks).unwrap();
    assert!(done.contains("- [x] "), "the fixture selects a ticked task: {done}");
    fs::write(&tasks, done.replacen("- [x] ", "- [ ] ", 1)).unwrap();

    let refusal = finalize_change(root, &record.id).unwrap_err();
    assert!(refusal.starts_with(OPEN_TASKS_REFUSAL), "{refusal}");
    let named = format!("then re-run `specsync change finalize {}`", record.id);
    assert!(refusal.ends_with(&named), "{refusal}");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted,
        "a refused retry archives nothing"
    );
    assert!(change_dir(root, &record.id).is_dir());

    fs::write(&tasks, &done).unwrap();
    let archive = finalize_change(root, &record.id).unwrap();
    assert!(archive.starts_with(root.join(ARCHIVE_PATH)), "{}", archive.display());
    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Archived);
}
