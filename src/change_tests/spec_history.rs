// #1084: a change's record in a module's history is its own fragment file
// (REQ-change-159, REQ-change-160, REQ-validator-048).

/// REQ-change-160: what `change check` writes parses back to the same change, date and title, and
/// a title with a line break is folded to one line so it always parses back.
#[test]
fn a_written_history_fragment_parses_back() {
    let content = history_fragment_content("add-passkeys", "2026-11-19", "Add\npasskeys");
    assert_eq!(
        content,
        "---\nchange: \"add-passkeys\"\ndate: 2026-11-19\n---\n\nAdd passkeys\n"
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
fn a_double_record_names_the_change_at_an_id_boundary() {
    let spec = "---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-18 | one-session: Hold one session |\n";
    assert!(changelog_records_change(spec, "one-session"));
    assert!(!changelog_records_change(spec, "session"));
    assert!(!changelog_records_change(spec, "one"));
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

/// #1086 review (Codex, spec_history.rs:73): every id `validate_change_id` accepts must round-trip
/// through the fragment `change check` writes, so `change:` is a quoted YAML string.
#[test]
fn a_fragment_quotes_its_change_id_so_every_valid_id_round_trips() {
    for id in ["true", "'foo'", "say \"hi\"", "a: b", "add-passkeys"] {
        let content = history_fragment_content(id, "2026-11-19", "Summary");
        assert!(
            content.starts_with("---\nchange: \""),
            "the id is written double-quoted: {content}"
        );
        let parsed = parse_history_fragment(&format!("{id}.md"), &content)
            .unwrap_or_else(|error| panic!("{id}: {error}\n{content}"));
        assert_eq!(parsed.change, id);
    }
    // Fragments written unquoted before this fix still read.
    assert_eq!(
        parse_history_fragment(
            "add-passkeys.md",
            "---\nchange: add-passkeys\ndate: 2026-11-19\n---\n\nSummary\n"
        )
        .unwrap()
        .change,
        "add-passkeys"
    );
}

/// #1086 review (Codex, spec_history.rs:327): one boundary-aware matcher decides both "this
/// module records the change" and "recorded twice". A reformatted row still names its change, and
/// a row for `one-session` does not name `session`.
#[test]
fn one_boundary_aware_matcher_finds_a_change_s_row() {
    let spec = |row: &str| {
        format!("---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n{row}\n")
    };
    let bold = spec("| 2026-11-18 | **add-passkeys:** Let a user sign in |");
    assert!(changelog_records_change(&bold, "add-passkeys"));
    let suffix = spec("| 2026-11-18 | one-session: Hold one session |");
    assert!(changelog_records_change(&suffix, "one-session"));
    assert!(
        !changelog_records_change(&suffix, "session"),
        "a row for one-session is not a row for session"
    );

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let spec_path = root.join("specs/auth/auth.spec.md");
    fs::create_dir_all(root.join("specs/auth/history")).unwrap();
    fs::write(&spec_path, &bold).unwrap();
    fs::write(
        history_fragment_path(&spec_path, "add-passkeys"),
        history_fragment_content("add-passkeys", "2026-11-19", "Let a user sign in"),
    )
    .unwrap();
    let (_, errors) = scan_history_fragments(root, &spec_path, &bold);
    assert!(
        errors.iter().any(|error| error.contains("records it too")),
        "a reformatted row plus a fragment is a double record: {errors:?}"
    );

    fs::write(&spec_path, &suffix).unwrap();
    fs::remove_file(history_fragment_path(&spec_path, "add-passkeys")).unwrap();
    fs::write(
        history_fragment_path(&spec_path, "session"),
        history_fragment_content("session", "2026-11-19", "Sessions"),
    )
    .unwrap();
    let (_, errors) = scan_history_fragments(root, &spec_path, &suffix);
    assert!(errors.is_empty(), "{errors:?}");
}

/// #1086 review (Codex, collect_requirement_ids.rs:2755): REQ-change-159 says materialization
/// never writes the spec's `## Change Log`, so a delta item that targets it is refused before
/// approval and before materialization, naming the item.
#[test]
fn a_delta_that_targets_the_change_log_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-19 | Rewritten |\n",
    );
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();

    let error = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap_err();

    assert!(error.contains("Change Log"), "{error}");
    assert!(error.contains("history/"), "the refusal says where history goes: {error}");
    assert_eq!(fs::read_to_string(&spec_path).unwrap(), spec);
    assert!(load_approvals(root, &record).unwrap().approvals.is_empty());
}

/// Codex's review of #1088: an id mentioned in another row's prose is not a record of that change,
/// so a valid fragment beside such a row is not a double record.
#[test]
fn an_id_in_another_row_s_prose_is_not_a_record() {
    let spec = "---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-18 | other-change: follows add-passkeys: and reuses its wording |\n";
    assert!(changelog_records_change(spec, "other-change"));
    assert!(!changelog_records_change(spec, "add-passkeys"));
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let spec_path = root.join("specs/auth/auth.spec.md");
    fs::create_dir_all(root.join("specs/auth/history")).unwrap();
    fs::write(&spec_path, spec).unwrap();
    fs::write(
        history_fragment_path(&spec_path, "add-passkeys"),
        history_fragment_content("add-passkeys", "2026-11-19", "Add passkeys"),
    )
    .unwrap();
    let (_, errors) = scan_history_fragments(root, &spec_path, spec);
    assert!(errors.is_empty(), "{errors:?}");

    // corvid-agent's review of #1088, its exact case: `session:` in a legacy row's prose, beside a
    // valid history/session.md, is not a double record.
    let legacy = "---\nversion: 3\n---\n\n## Change Log\n\n| Version | Date | Change |\n|---------|------|--------|\n| 3 | 2026-10-01 | one-session: Document session: handling |\n";
    fs::write(&spec_path, legacy).unwrap();
    fs::write(
        history_fragment_path(&spec_path, "session"),
        history_fragment_content("session", "2026-11-19", "Sessions"),
    )
    .unwrap();
    assert!(!changelog_records_change(legacy, "session"));
    let (_, errors) = scan_history_fragments(root, &spec_path, legacy);
    assert!(errors.is_empty(), "{errors:?}");
}

/// Codex's second review of #1088: an id that itself begins with `*`, `_` or a backtick is matched
/// as written, and only a marker pair that wraps the id is formatting, so `_draft:` is not a row
/// for `draft`.
#[test]
fn a_row_matches_the_raw_id_before_any_markdown_around_it() {
    let spec = |row: &str| {
        format!("---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n{row}\n")
    };
    assert!(changelog_records_change(&spec("| 2026 | *hotfix: Hot |"), "*hotfix"));
    assert!(!changelog_records_change(&spec("| 2026 | *hotfix: Hot |"), "hotfix"));
    assert!(changelog_records_change(&spec("| 2026 | _draft: Draft |"), "_draft"));
    assert!(!changelog_records_change(&spec("| 2026 | _draft: Draft |"), "draft"));
    // ss-lead's review of 8266e6c6: a whole cell in emphasis is a row, and the longest marker that
    // opens the cell is the one that must close, so `**hotfix: Hot**` is no row of `*hotfix`.
    assert!(changelog_records_change(&spec("| 2026 | **my-change: Title** |"), "my-change"));
    assert!(changelog_records_change(&spec("| 2026 | *hotfix: Hot* |"), "hotfix"));
    assert!(!changelog_records_change(&spec("| 2026 | **hotfix: Hot** |"), "*hotfix"));
    assert!(!changelog_records_change(&spec("| 2026 | **my-change: Title |"), "my-change"));
    for wrapped in ["**add-passkeys:**", "*add-passkeys:*", "__add-passkeys:__", "`add-passkeys:`"] {
        assert!(
            changelog_records_change(&spec(&format!("| 2026 | {wrapped} Passkeys |")), "add-passkeys"),
            "{wrapped}"
        );
    }
}

/// Codex's second review of #1088: a double-quoted `change:` reads YAML's escapes, so a fragment a
/// standard YAML tool wrote parses back to its id.
#[test]
fn a_quoted_change_id_reads_yaml_escapes() {
    for (written, id) in [("\"\\u0061\"", "a"), ("\"\\x61dd\"", "add"), ("\"a\\tb\"", "a\tb"), ("\"\\\"q\\\"\"", "\"q\"")] {
        assert_eq!(yaml_scalar(written).unwrap(), id, "{written}");
    }
    assert!(yaml_scalar("\"\\q\"").is_err());
    assert!(yaml_scalar("\"\\u12\"").is_err());
    let content = "---\nchange: \"\\u0061\"\ndate: 2026-11-19\n---\n\nSummary\n";
    assert_eq!(parse_history_fragment("a.md", content).unwrap().change, "a");
}

/// Codex's review of 8266e6c6 (the tester's case): a table cell's surrounding spaces are padding,
/// so an id that begins with a space is compared without it. A fragment for ` add-passkeys` beside
/// the legacy row `|  add-passkeys: Passkeys |` is a double record, not two separate records.
#[test]
fn a_leading_space_in_the_id_is_cell_padding_not_part_of_the_match() {
    let spec = "---\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-18 |  add-passkeys: Passkeys |\n";
    assert!(changelog_records_change(spec, " add-passkeys"));
    assert!(changelog_records_change(spec, "add-passkeys"));

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let spec_path = root.join("specs/auth/auth.spec.md");
    fs::create_dir_all(root.join("specs/auth/history")).unwrap();
    fs::write(&spec_path, spec).unwrap();
    fs::write(
        history_fragment_path(&spec_path, " add-passkeys"),
        history_fragment_content(" add-passkeys", "2026-11-19", "Passkeys"),
    )
    .unwrap();
    let (_, errors) = scan_history_fragments(root, &spec_path, spec);
    assert!(
        errors.iter().any(|error| error.contains("records it too")),
        "the row and the fragment are a double record: {errors:?}"
    );
}
