---
spec: generator.spec.md
---

## Key Decisions

- Generation is deterministic and local.
- Custom templates override built-ins without overwriting existing files.
- Companion creation and module discovery remain independent of coding-agent enrichment.
- CLI generate holds one project-root capability through template reads, directory creation, and
  no-overwrite publication; public-path checks detect replacement but never authorize writes.

## Files to Read First

- `src/generator.rs`
- `src/exports/mod.rs`
- `src/types.rs`

## Current Status

Stable local scaffold generator with no provider, credential, network, or shell path. CLI
publication is capability-relative and cannot follow a later public-root redirect.

This module owns what a generated scaffold looks like, and callers asking "has this module
recorded anything, or is it still the template" must ask here rather than keeping their own copy
of the prompt strings. `validator.rs` holds such a copy of three of the four context bullets and
it has already drifted — it omits the Notes bullet — which is the concrete cost of the second
copy rather than a hypothetical one.

A defect in scaffold handling is invisible to dogfooding on this repository: all 62 specs already
have authored prose, so no untouched scaffold exists here to trip over. Any affordance keyed on
"has the author written anything yet" needs a fixture that is the real generated artifact,
because the mature repository no longer contains one.
