// Issue #1024 (round 2, t384's review): a requirement whose canonical header carries a title
// suffix (`### REQ-cli-006 — Bound local input`) or punctuation (`### REQ-x-001: Title`) is
// readable by every requirement parser; the materializer now matches through the same shared
// parser and writes the matched header line back unchanged, so the title survives.

/// A MODIFIED delta lands on a titled requirement header, and the matched header line —
/// title included — is written back unchanged.
#[test]
fn a_modified_delta_matches_a_titled_requirement_header() {
    let source =
        "# Requirements\n\n### REQ-auth-001 — One session per user\n\nOld\n\n### REQ-auth-002\n\nKeep\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains("### REQ-auth-001 — One session per user\n\nNew"),
        "the title is kept: {modified}"
    );
    assert!(modified.contains("### REQ-auth-002\n\nKeep"));
}

/// The same id rule the validators use: a multi-segment module id with a title.
#[test]
fn a_modified_delta_matches_a_hyphenated_module_id_with_a_title() {
    let source = "# Requirements\n\n### REQ-cmd-change-012 — Titled\n\nOld\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-cmd-change-012",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains("### REQ-cmd-change-012 — Titled\n\nNew"),
        "{modified}"
    );
}

/// The parsers trim trailing punctuation from the id token, so a colon title matches too.
#[test]
fn a_modified_delta_matches_a_colon_title() {
    let source = "# Requirements\n\n### REQ-auth-001: One session per user\n\nOld\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains("### REQ-auth-001: One session per user\n\nNew"),
        "{modified}"
    );
}

/// The parser's ATX rule applies: up to three spaces of indent match; four is indented code.
#[test]
fn a_header_indented_three_spaces_matches_and_four_does_not() {
    let source = "# Requirements\n\n   ### REQ-auth-001 — Titled\n\nOld\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(modified.contains("   ### REQ-auth-001 — Titled\n\nNew"), "{modified}");

    let code = "# Requirements\n\n    ### REQ-auth-001 — Titled\n\nOld\n";
    let error = apply_markdown_block(code, "### ", "REQ-auth-001", "New", DeltaOperation::Modified)
        .unwrap_err();
    assert!(
        error.contains("cannot modify/remove missing block `REQ-auth-001`"),
        "four-space indent is indented code, not a heading: {error}"
    );
}

/// A REMOVED delta removes a titled requirement block.
#[test]
fn a_removed_delta_matches_a_titled_requirement_header() {
    let source =
        "# Requirements\n\n### REQ-auth-001 — One session per user\n\nOld\n\n### REQ-auth-002\n\nKeep\n";
    let removed = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "",
        DeltaOperation::Removed,
    )
    .unwrap();
    assert!(!removed.contains("REQ-auth-001"));
    assert!(removed.contains("### REQ-auth-002\n\nKeep"));
}

/// The id token boundary holds: `REQ-auth-001` never matches `REQ-auth-0010`.
#[test]
fn a_titled_match_never_crosses_the_id_token_boundary() {
    let source = "# Requirements\n\n### REQ-auth-0010 — Another block\n\nOld\n";
    let error = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap_err();
    assert!(
        error.contains("cannot modify/remove missing block `REQ-auth-001`"),
        "a longer id is not a titled match: {error}"
    );
}

/// An ADDED item whose id already exists behind a titled header with the same body converges
/// instead of duplicating the block (t384's round-2 edge).
#[test]
fn an_added_delta_behind_a_titled_header_converges() {
    let source = "# Requirements\n\n### REQ-auth-001 — One session\n\nSame body.\n";
    let reapplied = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "Same body.",
        DeltaOperation::Added,
    )
    .unwrap();
    assert_eq!(
        reapplied.matches("REQ-auth-001").count(),
        1,
        "no duplicate block: {reapplied}"
    );
    // A different body behind the title is a conflict, not a silent no-op.
    let error = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "Different body.",
        DeltaOperation::Added,
    )
    .unwrap_err();
    assert!(error.contains("cannot add existing block"), "{error}");
}

/// `delta_item_is_applied` recognizes an already-applied delta whose canonical
/// block still carries its title, so convergence checks see the applied state.
#[test]
fn an_applied_delta_is_recognized_behind_a_titled_header() {
    let requirements =
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001 — One session\n\nNew body.\n";
    let item = DeltaItem {
        operation: DeltaOperation::Modified,
        target: DeltaTarget::Requirement,
        key: "REQ-auth-001".to_string(),
        content: "New body.".to_string(),
    };
    assert!(delta_item_is_applied("", requirements, &item));
}

/// A `## ` section key is matched exactly: `Change Log` never matches
/// `## Change Log Archive`.
#[test]
fn a_section_key_never_matches_a_titled_section_heading() {
    let source = "# Spec\n\n## Change Log Archive\n\nOld rows.\n";
    let error = apply_markdown_block(
        source,
        "## ",
        "Change Log",
        "New rows.",
        DeltaOperation::Modified,
    )
    .unwrap_err();
    assert!(
        error.contains("cannot modify/remove missing block `Change Log`"),
        "a section heading with a suffix is not a match: {error}"
    );
}

/// t384's round-3 finding: an indented header ends the matched block too. A MODIFIED on an
/// indented header must not run through (and delete) the next indented requirement.
#[test]
fn an_indented_header_ends_the_matched_block() {
    let source = "# Requirements\n\n   ### REQ-auth-001 — One\n\nOld\n\n   ### REQ-auth-002 — Two\n\nKeep\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains("   ### REQ-auth-002 — Two\n\nKeep"),
        "the next indented requirement survives: {modified}"
    );
    assert!(modified.contains("   ### REQ-auth-001 — One\n\nNew"));

    let removed = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "",
        DeltaOperation::Removed,
    )
    .unwrap();
    assert!(
        removed.contains("   ### REQ-auth-002 — Two\n\nKeep"),
        "REMOVED does not eat the next block: {removed}"
    );
}

/// The kept header line is byte-for-byte, trailing spaces included (the AC says so).
#[test]
fn the_kept_header_keeps_its_trailing_spaces() {
    let source = "# Requirements\n\n### REQ-auth-001 — Titled   \n\nOld\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains("### REQ-auth-001 — Titled   \n\nNew"),
        "trailing spaces survive: {modified:?}"
    );
}
