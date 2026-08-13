---
change: CHG-0110-report-the-specs-that-dependency-analysis-dropped-instead-of-calling-a-malformed
artifact: testing
---

# Testing

## Strategy

The defect is that a command reports success over input it could not read. So the pair that
matters is: the malformed case must now fail **and name the cause**, while the well-formed
case must still pass. Asserting only the first would be satisfied by a command that failed
on everything.

## Verified by hand

| fixture | before | after |
|---|---|---|
| `depends_on: {oops` | `✓ All dependency declarations are valid`, exit 0 | `✗ specs/bad/bad.spec.md: Frontmatter field \`depends_on\` must be a YAML list, got a mapping (offending line: \`depends_on: {oops\`)`, exit 1 |
| **control** — well-formed spec, `depends_on: []` | `✓ … valid`, exit 0 | unchanged: `✓ … valid`, exit 0 |

The message is the validator's own wording, so `check` and `deps` now agree on identical
input rather than contradicting each other.

## Regression surface

The change adds reports where there were silent `continue`s and leaves the well-formed path
untouched. The suite is the guard against reporting too eagerly — 2210 unit and 331
integration tests pass unchanged, including this repository's own 62 specs, all of which
parse.

## Not covered

The two secondary drops — unparseable frontmatter, and a spec declaring no `module` — were
found while fixing the filed defect and are reasoned rather than fixture-tested here. Both
follow the identical code path as the primary case (append to the same error list, which is
what makes the command exit non-zero), so the primary fixture exercises the mechanism. A
dedicated fixture belongs in the sandbox alongside the existing `deps` coverage.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-deps-002 | `cargo test` (2210 + 331, 0 failures) plus the hand-verified pair above: the malformed declaration is reported with the validator's exact wording and exits 1, and the well-formed control still reports a valid graph and exits 0. The two secondary drops append to the same error list, so they share the mechanism the fixture proves |
