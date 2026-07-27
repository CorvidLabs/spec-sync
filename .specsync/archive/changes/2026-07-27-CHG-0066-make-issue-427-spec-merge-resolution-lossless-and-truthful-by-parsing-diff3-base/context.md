---
change: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
artifact: context
---

# Context

## Baseline Behavior Before CHG-0066

The baseline merge implementation used side-precedence shortcuts for scalar fields and generic
table rows. It did not model a diff3 base region as a distinct input, and some status messages
could attribute incoming content to HEAD.

## Decision

Use conservative, content-aware resolution. Known lossless shapes may resolve automatically;
any ambiguous, malformed, orphan, nested, or lossy-parser-boundary region makes the entire file
manual and prevents writes. Candidate frontmatter is checked for YAML validity, duplicate keys,
required fields, valid status, and non-empty files. This matches the repository's safety
invariant and the delivery plan approved for issue #427.

## Scope

- Implementation: `src/merge.rs`
- CLI integration coverage: `tests/integration/commands.rs`
- Canonical contract and companions: `specs/merge/`
- No CLI flags or exported Rust symbol names change.

## Delivery

Reconstruct PR #448 from current `main` so the unsafe intermediate implementation is not replayed.
The final PR metadata must describe all-or-nothing persistence, not partial writes.

Independent acceptance and adversarial reviews found additional pre-existing paths through which
headers, nested YAML, malformed markers, or one-sided scalars could be rewritten. CHG-0066 treats
those findings as blockers and includes their regression coverage. A second adversarial pass also
identified header-only hunks, YAML null-versus-empty-list coercion, missing-frontmatter writes, and
premature `Auto-resolvable` wording after persistence; all four paths now have explicit regression
coverage and passed fresh independent adversarial confirmation.
