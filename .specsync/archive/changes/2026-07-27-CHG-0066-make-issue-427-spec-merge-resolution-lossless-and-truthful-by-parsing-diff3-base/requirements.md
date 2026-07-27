---
change: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
artifact: requirements
---

# Requirements

## Problem

Issue #427 demonstrates that the merge engine can misinterpret diff3 base regions, choose the
wrong scalar or table value, and describe the selected side inaccurately. A partly resolvable
file must not be rewritten because doing so can destroy the user's unresolved conflict context.

## Required Outcomes

- Parse standard two-way and diff3 conflict regions without treating base-ancestor bytes as an
  editable side.
- Preserve both side labels in truthful diagnostics.
- Union supported list fields and select the maximum numeric frontmatter version.
- Leave same-key table disagreements and divergent or non-numeric scalar fields unresolved.
- Reject malformed regions and reconstructed frontmatter that fails canonical parsing.
- Preserve CRLF and final-newline form.
- Perform an all-or-nothing write: one manual region leaves the complete original file unchanged.

## Compatibility

- Existing lossless changelog and list-field resolutions remain automatic.
- Public Rust exports remain unchanged.
- Ambiguous cases become conservative manual results instead of silently choosing content.
