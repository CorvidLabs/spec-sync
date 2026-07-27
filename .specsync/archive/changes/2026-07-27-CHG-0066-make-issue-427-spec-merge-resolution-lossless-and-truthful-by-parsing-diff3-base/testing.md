---
change: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
artifact: testing
---

# Testing

## Characterization

- Reproduce a diff3 region whose base bytes must not appear in resolved output.
- Reproduce divergent same-key table rows and scalar fields that previously selected one side.
- Reproduce a mixed file containing one resolvable region and one manual region, then compare the
  complete file byte-for-byte before and after.

## Targeted Regression Coverage

- `REQ-merge-001` maps to the focused merge tests below and requires a passing lifecycle
  verification manifest before acceptance.
- Two-way and diff3 parser behavior, including both labels.
- Numeric version maximum and supported list unions.
- Table-row and scalar conflicts remaining manual.
- Exact marker grammar plus malformed, orphan, duplicate, nested, incomplete, and lookalike forms.
- Table headers with the separator inside or immediately after the hunk, nested mappings,
  null-versus-empty-list disagreements, one-sided scalars, missing frontmatter, and invalid
  reconstructed frontmatter.
- Unreadable all-files candidates and failed Git discovery.
- CRLF input and files both with and without a final newline.
- Dry-run and real-run all-or-nothing persistence.
- Human output side attribution, `Auto-resolvable` dry-run wording, and `Auto-resolved` wording
  only after a successful write. Structured JSON dry-run schema is owned by issue #420.

## Required Gates

- `cargo test merge::`
- `fledge lanes run verify`
- Full repository lane
- Strict 100% spec coverage and score at least 80
- `fledge trust verify`
- Independent acceptance-row review and adversarial regression review
