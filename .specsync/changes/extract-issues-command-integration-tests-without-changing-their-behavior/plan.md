---
change: extract-issues-command-integration-tests-without-changing-their-behavior
artifact: plan
---

# Plan

## Approach

Move only the contiguous issues integration-test family into a real Rust module, preserving all test semantics. Keep shared parent helpers in place and update the existing testing companion's file paths and filters.

## Steps

1. Inventory and hash the original 26 tests, attributes and fixture bytes.
2. Record the supported-host 24-test baseline, then extract the exact block.
3. Verify independent byte conservation of the entire moved core and remaining process file, shared helpers and environment isolation.
4. Run the moved family, full native lane, release integration compile, latest released strict check and mandatory pre-push gate.
5. Publish a draft with truthful unsigned agent provenance, documented platform limits and corvid-agent review requested.
6. Record the observed lifecycle CI requirement in this complete but unapproved draft. Do not approve, accept, review or finalize as a human.

## Out of scope

No product source or canonical contract change, altered assertions, ignored tests, fixture replacement, CI/policy/protection edits, other PR-owned paths, dependency downloads, releases or merges.

## Risks

A move can silently drop tests or alter cfg/privacy/path resolution. The full moved core and remaining file are independently byte-compared; supported test-name sets are identical before/after. The Windows helper import matches its unchanged cfg. Linux/Windows-only cases remain source-reviewed, not executed on this Mac.

## Constraints consulted

Repository/parent AGENTS, existing integration layout/helpers, cmd_issues testing companion, native Fledge lanes, unchanged Trust/Attest settings and the actual hosted lifecycle failure.
