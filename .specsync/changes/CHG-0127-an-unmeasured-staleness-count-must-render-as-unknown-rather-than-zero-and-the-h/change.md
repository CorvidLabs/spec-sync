---
id: CHG-0127-an-unmeasured-staleness-count-must-render-as-unknown-rather-than-zero-and-the-h
state: implementing
type: bug_fix
base_commit: 16013babe407bc096f403c0fa9833c56a14533e2
---

# An unmeasured staleness count must render as unknown rather than zero, and the hand-rolled config scanner must report a malformed header rather than silently skipping it

## Intent

An unmeasured staleness count must render as unknown rather than zero, and the hand-rolled config scanner must report a malformed header rather than silently skipping it

## Affected Canonical Specs

- `cmd_report`
- `config`

## Acceptance Criteria

- On a tree whose staleness cannot be measured, report renders the stale count as unknown in text and null in JSON, never zero, because zero is an answer and there is none. A tree with real git history reports its actual count unchanged. A config file whose header is unterminated is reported as unloadable rather than silently skipped, using the same refusal wording as the unreadable-file shape, so a consumer matching a refusal need not know which door it came through. A well-formed config is unaffected, and valid TOML the scanner does not implement is not rejected.

## No-spec Rationale

Not applicable
