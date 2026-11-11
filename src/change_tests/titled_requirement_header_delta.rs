// Issue #1024: a requirement whose canonical header carries a title suffix
// (`### REQ-cli-006 — Bound local input`) is readable by every requirement
// parser, which all take the first whitespace-separated token, but the delta
// materializer matched headings by exact line equality, so a MODIFIED or
// REMOVED delta naming the id failed with "cannot modify/remove missing
// block". Requirement blocks now match by id and tolerate the title; `## `
// section keys keep exact matching.

/// A MODIFIED delta lands on a titled requirement header, and the
/// materialized block carries the canonical untitled header.
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
        modified.contains("### REQ-auth-001\n\nNew"),
        "the title is normalized away so re-application converges: {modified}"
    );
    assert!(modified.contains("### REQ-auth-002\n\nKeep"));
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

/// The block a titled match returns runs to the next heading, not to the end
/// of the line, and the id token boundary holds: `REQ-auth-001` never matches
/// `REQ-auth-0010`.
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
