---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: research
---

# Research

## Evidence reviewed

- The recursive-verifier classifier recognizes direct SpecSync and root-manifest Cargo selection but does not currently honor Cargo's `--manifest-path` option.
- Registry-derived coverage currently compares only the exact canonical spec and `requirements.md`; broadening to the containing directory would incorrectly cover unrelated siblings.
- The interview's shared `split_values` helper is applied before question dispatch, so prose containing commas or newlines is silently converted into multiple acceptance criteria.
- Existing lifecycle tests already exercise root-manifest Cargo identity, registry mappings, protected inputs, and deterministic rendering; the new tests extend those seams rather than introducing a parallel mechanism.

## Decision

Use explicit syntax for multiple prose values and precise allowlists for implicit file coverage. Both choices favor predictable behavior and fail-closed governance without making the common single-user workflow more verbose.
