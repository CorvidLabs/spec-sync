// #1084: a change's record in a module's history is its own fragment file
// (REQ-change-159, REQ-change-160, REQ-validator-048).

/// REQ-change-160: what `change check` writes parses back to the same change, date and title, and
/// a title with a line break is folded to one line so it always parses back.
#[test]
fn a_written_history_fragment_parses_back() {
    let content = history_fragment_content("add-passkeys", "2026-11-19", "Add\npasskeys");
    assert_eq!(
        content,
        "---\nchange: add-passkeys\ndate: 2026-11-19\n---\n\nAdd passkeys\n"
    );
    assert_eq!(
        parse_history_fragment("add-passkeys.md", &content).unwrap(),
        HistoryFragment {
            change: "add-passkeys".into(),
            date: "2026-11-19".into(),
            summary: "Add passkeys".into(),
        }
    );
}

/// REQ-validator-048: each malformed fragment is refused with a reason.
#[test]
fn a_malformed_history_fragment_is_refused_with_its_reason() {
    let good = "---\nchange: a\ndate: 2026-02-28\nnote: other keys are ignored\n---\n\nSummary\n";
    assert!(parse_history_fragment("a.md", good).is_ok());
    assert!(
        parse_history_fragment("a.md", &good.replace('\n', "\r\n")).is_ok(),
        "CRLF line endings parse"
    );
    for (name, content, reason) in [
        ("a.txt", good, "named `<change-id>.md`"),
        ("b.md", good, "named for `b`"),
        ("a.md", "Summary\n", "does not open with `---`"),
        ("a.md", "---\nchange: a\ndate: 2026-02-28\n", "no closing `---`"),
        ("a.md", "---\ndate: 2026-02-28\n---\n\nS\n", "no `change:`"),
        ("a.md", "---\nchange: a\n---\n\nS\n", "no `date:`"),
        ("a.md", "---\nchange: a\nchange: a\ndate: 2026-02-28\n---\n\nS\n", "twice"),
        ("a.md", "---\nchange: a\ndate: 2026-02-30\n---\n\nS\n", "not a YYYY-MM-DD date"),
        ("a.md", "---\nchange: a\ndate: 2026-13-01\n---\n\nS\n", "not a YYYY-MM-DD date"),
        ("a.md", "---\nchange: a\ndate: 26-1-1\n---\n\nS\n", "not a YYYY-MM-DD date"),
        ("a.md", "---\nchange: a\ndate: 2026-02-28\n---\n\n", "no summary"),
        ("a.md", "---\nchange: a\ndate: 2026-02-28\n---\n\nOne\nTwo\n", "more than one line"),
    ] {
        let error = parse_history_fragment(name, content).unwrap_err();
        assert!(error.contains(reason), "{name}: {error}");
    }
    assert!(is_calendar_date("2024-02-29"));
    assert!(!is_calendar_date("2026-02-29"));
}

/// REQ-validator-048: a double record is a Change Log cell that starts with `<id>:`, not any text
/// that ends with it, so a fragment for `session` is not claimed by a row for `one-session`.
#[test]
fn a_double_record_is_a_change_log_cell_that_starts_with_the_change_id() {
    let spec = "---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-18 | one-session: Hold one session |\n";
    assert!(changelog_row_names_change(spec, "one-session"));
    assert!(!changelog_row_names_change(spec, "session"));
    assert!(!changelog_row_names_change(spec, "one"));
}

/// REQ-change-160: fragments order by date, then change id, whatever their file names.
#[test]
fn history_fragments_order_by_date_then_change_id() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let spec_path = root.join("specs/auth/auth.spec.md");
    fs::create_dir_all(root.join("specs/auth/history")).unwrap();
    for (id, date) in [("b", "2026-11-02"), ("a", "2026-11-02"), ("z", "2026-11-01")] {
        fs::write(
            history_fragment_path(&spec_path, id),
            history_fragment_content(id, date, "Summary"),
        )
        .unwrap();
    }
    let (fragments, errors) = scan_history_fragments(root, &spec_path, "");
    assert!(errors.is_empty(), "{errors:?}");
    let order: Vec<&str> = fragments.iter().map(|f| f.change.as_str()).collect();
    assert_eq!(order, ["z", "a", "b"]);
}

/// REQ-change-159 and REQ-change-092: materialization records the change in a fragment and never
/// writes `version:` or the Change Log; a second run writes nothing; the fragment is one of the
/// change's own files for `check --commit`.
#[test]
fn materialization_records_the_change_once_in_a_fragment() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nOne session per user.\n",
    );
    let spec_path = root.join("specs/auth/auth.spec.md");
    let before = fs::read_to_string(&spec_path).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    materialize_change_deltas(root, &record.id).unwrap();

    let after = fs::read_to_string(&spec_path).unwrap();
    assert!(after.contains("One session per user."), "{after}");
    assert_eq!(
        after.replace("One session per user.", "Stable."),
        before,
        "only the delta's own text changes: no version bump, no Change Log row"
    );
    let fragment_path = history_fragment_path(&spec_path, &record.id);
    let fragment = fs::read_to_string(&fragment_path).unwrap();
    assert_eq!(
        fragment,
        history_fragment_content(&record.id, &today(), &record.title)
    );

    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(fs::read_to_string(&spec_path).unwrap(), after);
    assert_eq!(fs::read_to_string(&fragment_path).unwrap(), fragment);

    let scope = lifecycle_commit_scope(root, &record.id).unwrap();
    assert!(
        scope
            .owned
            .contains(&format!("specs/auth/history/{}.md", record.id)),
        "{:?}",
        scope.owned
    );
}
