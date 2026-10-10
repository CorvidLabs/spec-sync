---
change: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
artifact: context
---

# Context

## Scope and authorization

The owner asked to break up large spec-sync files, continue on spec-sync and make a branch and commit. This package applies that authorization to the complete `src/commands/change.rs` file without changing behavior. corvid-agent records the owner's scope authorization, not a human implementation review. Implementation review, shipping and publication remain pending.

## Architecture

The command facade continues to dispatch the existing public API while private modules own rendering, ship-status, archive guidance, Git inspection, shipping, owner-input resolution and lifecycle commits. Domain policy stays in `crate::change`. All existing tests and guards move intact to topical modules with shared fixtures.
