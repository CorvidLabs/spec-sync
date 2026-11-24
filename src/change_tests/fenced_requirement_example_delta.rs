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

    #[test]
    fn a_removed_requirement_is_applied_when_only_a_fenced_example_remains() {
        let requirements = "# Requirements\n\n```markdown\n### REQ-auth-001\n\nExample\n```\n\n### REQ-auth-002\n\nKeep\n";
        let item = DeltaItem {
            operation: DeltaOperation::Removed,
            target: DeltaTarget::Requirement,
            key: "REQ-auth-001".into(),
            content: String::new(),
        };
        assert!(delta_item_is_applied("", requirements, &item));
    }

    #[test]
    fn a_modified_requirement_is_applied_when_its_real_block_carries_the_new_body() {
        let requirements = "# Requirements\n\n```markdown\n### REQ-auth-001\n\nExample\n```\n\n### REQ-auth-001\n\nNew\n\n### REQ-auth-002\n\nKeep\n";
        let item = DeltaItem {
            operation: DeltaOperation::Modified,
            target: DeltaTarget::Requirement,
            key: "REQ-auth-001".into(),
            content: "New".into(),
        };
        assert!(delta_item_is_applied("", requirements, &item));
    }

    #[test]
    fn a_backtick_in_an_info_string_keeps_the_next_heading_visible() {
        let prefix = "# Contract\n\n```markdown `example`\n\n";
        for (level, key) in [("### ", "REQ-auth-001"), ("## ", "Purpose")] {
            let heading = format!("{level}{key}");
            let source = format!("{prefix}{heading}\n\nOld\n\n## Keep\n\nText\n");
            let modified =
                apply_markdown_block(&source, level, key, "New", DeltaOperation::Modified).unwrap();
            assert_eq!(
                modified,
                format!("{prefix}{heading}\n\nNew\n\n## Keep\n\nText\n"),
            );
        }
    }
}
