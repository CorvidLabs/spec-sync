---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: requirements
---

# Requirements

## REQ-commands-006 — a draft that claims a contract over present source is reported

A spec with `status: draft` SHALL produce a warning when both of the following hold:

1. at least one of its mapped source files was present and readable; and
2. its Public API section names at least one symbol.

A draft spec that satisfies neither, or only one, SHALL NOT produce that warning. In
particular a draft whose mapped files do not exist yet SHALL continue to pass strict
validation unchanged, and so SHALL a draft whose Public API names no symbol.

Bare `specsync check` SHALL remain exit 0 in every draft case; only `--strict` gates.

## REQ-types-005 — validation records what it was able to observe

`ValidationResult` SHALL record whether any mapped source file was present, and whether
the spec's Public API names at least one symbol, so that a reporter can distinguish a
check that passed from a check that did not run.

Both SHALL be recorded even when section and export validation are skipped.

## REQ-validator-011 — a directory or absent mapping is not a present source

A mapped source file SHALL be recorded as present only when it resolved to a readable
file. A mapping that is missing, planned, a directory, unreadable, or rejected for
escaping the project root SHALL NOT be recorded as present.

## REQ-hash-cache-002 — drift is reported only against a baseline

A change classification SHALL record whether the cache held a prior hash for the spec.

An absent cache entry SHALL continue to select the spec for re-validation, and SHALL NOT
be reported as a change. A companion change observed against a known baseline SHALL
continue to be reported.

The cache's own frontmatter `files:` extraction SHALL resolve a quoted entry to the same
path the parser resolves.

## REQ-cmd-check-005 — staleness output requires evidence

`specsync check` SHALL report requirements drift and companion updates only for
classifications observed against a known baseline, in every output format.

Spec selection SHALL be unaffected: the same specs are re-validated either way.

## REQ-parser-002 — quoted frontmatter scalars and list items are unquoted

Frontmatter parsing SHALL resolve a single- or double-quoted block list item or scalar to
the text inside the quotes, for every field.

A comment following the closing quote SHALL be discarded, and a `#` inside the quotes SHALL
be retained as content.

An opening quote with no matching close SHALL be a frontmatter error naming the offending
value, and the value SHALL NOT be retained as a literal.

Flow-style lists SHALL continue to unquote their own items.

## REQ-change-064 — the coverage remediation stays readable

The uncovered-paths remediation SHALL name at most a fixed number of paths explicitly and
SHALL summarize the remainder with a count and a covering-prefix suggestion.
