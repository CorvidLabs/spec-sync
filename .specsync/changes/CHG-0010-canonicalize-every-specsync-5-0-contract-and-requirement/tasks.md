---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: tasks
---

# Tasks

- [x] Reduce affected scope from 62 specs to the exact 53-spec audit union.
- [x] Repair acceptance criteria and complete definition artifacts.
- [x] Draft one valid semantic delta for every affected module.
- [x] Update 35 canonical `depends_on` frontmatter edges during implementation.
- [x] Draft nine Public API signature corrections and current configuration/rule prose for atomic acceptance.
- [x] Add 44 stable normative requirement identities without removing legacy detail.
- [x] Reconcile and promote `cmd_migrate` after focused verification.
- [x] Add missing `cmd_rules/context.md` frontmatter.
- [x] Run every locally available gate in `testing.md` and record evidence.
- [x] Ignore Rust imports embedded in non-code and resolve source-module ownership.
- [x] Remove the commands/rehash architectural dependency cycle without changing CLI behavior.
- [x] Re-run dependency discovery and remove any declarations proven to be false edges.
- [x] Correct stale testing claims for Lua export support, rehash inline tests, and cmd_new integration fixtures.

## Release Gates

Closing acceptance remains blocked on the corrected Linux, macOS, and Windows PR matrix. After acceptance, rerun the
canonical strict check and score, require the post-acceptance matrix, merge, and require post-merge `main` before
release. Archive only after delivery integration is proven.
