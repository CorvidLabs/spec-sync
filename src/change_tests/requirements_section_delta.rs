// Issue #866 (REQ-change-162): a `### REQUIREMENTS SECTION <name>` delta item adds, replaces or
// removes a whole `## <name>` section of `requirements.md`, such as `## Acceptance Criteria` or
// `## Constraints`, as `### SPEC SECTION <name>` does for the canonical spec. A section ends at
// the next heading of level 2 or shallower or at the next requirement heading, so a requirement
// block is never part of a section, and an item whose content holds a requirement heading is
// refused, so requirements change only through `### REQUIREMENT` items.

mod requirements_section_delta {
    use super::*;

const REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n## User Stories\n\n- As a user, I sign in.\n\n## Acceptance Criteria\n\n- Old criterion.\n\n## Out of Scope\n\n- Billing.\n\n### REQ-auth-001\n\nThe system SHALL keep one session.\n\nAcceptance Criteria\n- One session.\n";

const REQUIREMENT_BLOCK: &str =
    "### REQ-auth-001\n\nThe system SHALL keep one session.\n\nAcceptance Criteria\n- One session.\n";

fn only_item(delta: &str) -> DeltaItem {
    let mut items = parse_delta(delta).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(items.len(), 1, "{delta}");
    items.remove(0)
}

/// `requirements` after applying the one item in `delta`; the spec text is untouched.
fn applied_to(requirements: &str, delta: &str) -> Result<String, String> {
    let item = only_item(delta);
    let (spec, requirements) = apply_delta_item("spec\n", requirements, &item, item.operation)?;
    assert_eq!(spec, "spec\n", "a requirements section never touches the spec");
    Ok(requirements)
}

#[test]
fn the_heading_parses_case_insensitively() {
    for heading in ["REQUIREMENTS SECTION", "requirements section"] {
        let item = only_item(&format!("## MODIFIED\n\n### {heading} Constraints\n\n- Fast.\n"));
        assert_eq!(item.key, "Constraints");
    }
}

#[test]
fn a_modified_section_replaces_the_whole_list_and_nothing_else() {
    let delta = "## MODIFIED\n\n### REQUIREMENTS SECTION Acceptance Criteria\n\n- New one.\n- New two.\n";
    let out = applied_to(REQUIREMENTS, delta).unwrap();
    assert!(
        out.contains("## Acceptance Criteria\n\n- New one.\n- New two.\n"),
        "{out}"
    );
    assert!(!out.contains("Old criterion"), "{out}");
    assert!(out.contains("## User Stories\n\n- As a user, I sign in.\n"), "{out}");
    assert!(out.contains("## Out of Scope\n\n- Billing.\n"), "{out}");
    assert!(out.ends_with(REQUIREMENT_BLOCK), "{out}");
    assert_eq!(applied_to(&out, delta).unwrap(), out, "re-applying converges");
}

#[test]
fn a_section_followed_by_requirement_blocks_ends_at_the_first_of_them() {
    let out = applied_to(
        REQUIREMENTS,
        "## MODIFIED\n\n### REQUIREMENTS SECTION Out of Scope\n\n- Billing and refunds.\n",
    )
    .unwrap();
    assert!(out.contains("## Out of Scope\n\n- Billing and refunds.\n"), "{out}");
    assert!(out.ends_with(REQUIREMENT_BLOCK), "the requirement is kept: {out}");
}

#[test]
fn removed_drops_only_that_section() {
    let out = applied_to(
        REQUIREMENTS,
        "## REMOVED\n\n### REQUIREMENTS SECTION Out of Scope\n",
    )
    .unwrap();
    assert!(!out.contains("## Out of Scope") && !out.contains("Billing"), "{out}");
    assert!(out.contains("## Acceptance Criteria\n\n- Old criterion.\n"), "{out}");
    assert!(out.ends_with(REQUIREMENT_BLOCK), "{out}");
}

#[test]
fn added_appends_a_missing_section_and_refuses_one_with_other_content() {
    let out = applied_to(
        REQUIREMENTS,
        "## ADDED\n\n### REQUIREMENTS SECTION Constraints\n\n- Under 100 ms.\n",
    )
    .unwrap();
    assert!(out.contains("## Constraints\n\n- Under 100 ms."), "{out}");
    assert!(out.contains(REQUIREMENT_BLOCK), "{out}");
    let same = applied_to(
        REQUIREMENTS,
        "## ADDED\n\n### REQUIREMENTS SECTION Acceptance Criteria\n\n- Old criterion.\n",
    )
    .unwrap();
    assert_eq!(same, REQUIREMENTS, "an ADDED section already present converges");
    let error = applied_to(
        REQUIREMENTS,
        "## ADDED\n\n### REQUIREMENTS SECTION Acceptance Criteria\n\n- Different.\n",
    )
    .unwrap_err();
    assert!(error.contains("MODIFIED"), "{error}");
}

#[test]
fn an_item_holding_a_requirement_heading_is_refused() {
    let error = requirements_section_item_problem(&only_item(
        "## MODIFIED\n\n### REQUIREMENTS SECTION Out of Scope\n\n- Billing.\n\n#### Note\n\n### REQ-auth-009\n\nThe system SHALL do more.\n",
    ))
    .expect("an item holding a requirement heading is refused");
    assert!(error.contains("REQ-auth-009"), "{error}");
    assert!(error.contains("### REQUIREMENT"), "{error}");
    // A fenced example is quoted text, not a requirement.
    assert_eq!(
        requirements_section_item_problem(&only_item(
            "## MODIFIED\n\n### REQUIREMENTS SECTION Notes\n\n```markdown\n### REQ-auth-009\n```\n",
        )),
        None
    );
}
}
