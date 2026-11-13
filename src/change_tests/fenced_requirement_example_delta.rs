// Issue #1050 (t384's review of #1049): the shared requirement reader treats a fenced example
// as quoted text, never a requirement (`validator::requirement_ids`), and REQ-change-152 has the
// materializer match a block the way that reader does. So a fenced example holding a
// requirement's header above the real block is never the block a MODIFIED delta replaces, and
// a heading inside a fenced example within the real block never ends that block.

mod fenced_requirement_example_delta {
    use super::*;

/// A MODIFIED delta replaces the real block, not a fenced example of the same id above it,
/// and the example stays byte for byte.
#[test]
fn a_modified_delta_skips_a_fenced_example_of_the_same_id() {
    let example = "```markdown\n### REQ-auth-001\n\nExample text\n```\n";
    let source = format!(
        "# Requirements\n\nA delta looks like this:\n\n{example}\n### REQ-auth-001\n\nOld\n\n### REQ-auth-002\n\nKeep\n"
    );
    let modified = apply_markdown_block(
        &source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        modified.contains(example),
        "the fenced example is kept: {modified}"
    );
    assert!(
        !modified.contains("Old"),
        "the real block is replaced: {modified}"
    );
    assert!(modified.contains("### REQ-auth-001\n\nNew"), "{modified}");
    assert!(modified.contains("### REQ-auth-002\n\nKeep"), "{modified}");
}

/// A heading inside a fenced example within the real block does not end the block, so a
/// MODIFIED delta replaces all of it.
#[test]
fn a_fenced_heading_inside_the_block_does_not_end_it() {
    let source = "# Requirements\n\n### REQ-auth-001\n\nOld\n\n```markdown\n## Example\n```\n\nStill old\n\n### REQ-auth-002\n\nKeep\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(
        !modified.contains("Still old") && !modified.contains("## Example"),
        "the whole block, fenced example included, is replaced: {modified}"
    );
    assert!(modified.contains("### REQ-auth-001\n\nNew"), "{modified}");
    assert!(modified.contains("### REQ-auth-002\n\nKeep"), "{modified}");
}
}
