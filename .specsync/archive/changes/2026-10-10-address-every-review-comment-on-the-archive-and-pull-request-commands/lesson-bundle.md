# Lesson bundle — address-every-review-comment-on-the-archive-and-pull-request-commands

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Address every review comment on the archive and pull request commands
- **Kind**: BugFix
- **Specs**: change, cmd_change, cmd_rules
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/commands/rules.rs, .github/workflows/ci.yml, .github/scripts/archive-readiness.sh, .github/scripts/test-archive-readiness.sh, .github/scripts/test-required-ci-gate.py, specs/change/change.spec.md, specs/change/requirements.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_rules/cmd_rules.spec.md, specs/cmd_rules/requirements.md
- **Acceptance**: Every review comment on pull request 804 is fixed in the product, and each thread can be resolved.

## Evidence

- Verification commit: `77aac0ab1cf1cd893688c6e9ad1de483e89d718a`
- Base commit: `44eb74e2981f125c5dc57ce8a0c6fa3e3923e53c`
- Verified by: `specsync check --spec change --spec cmd_change --spec cmd_rules`

## From the change's context.md

# Context

Pull request 804 gathered 24 bot review threads. The product checks were green and merge was blocked because every thread was unresolved. The threads name real defects in archive ordering, `change pr`, plan completeness, lesson paths, and the implementation gate.

## From the change's design.md

# Design

`change pr` opens or reuses an open pull request even when the change is not ready to archive, and it archives only when this change is ready and first. It runs the pre-push gate before pushing and prints one result. Merge guidance is the archive sentence: merge only when nothing is open.

The Archive check and the Rust ranking share one lightweight proof for recommending ship: workflow v2, verdict pass, and the reviewed commit equal to HEAD. Accepted ranks ahead of verifying. An archived state left under `.specsync/changes/` stays open.

Archive stays out of SpecSync implementation ready and stays on the required merge gate.

## From the change's testing.md

# Testing

- `cargo test --bin specsync` covers the plan-section, lesson-comment, and archive-ranking tests for REQ-change-105 and REQ-change-106.
- `.github/scripts/test-archive-readiness.sh` covers accepted ranking, a blocking review, and an archived state left in the active tree for REQ-cmd-change-019.
- `python3 .github/scripts/test-required-ci-gate.py` covers Archive failing the required gate without failing implementation ready for REQ-cmd-change-019.
- REQ-cmd-change-020 is the `change pr` contract: one JSON result, an open pull request is reused, a closed one is not, and merge is not said while a change is open.
- REQ-cmd-rules-002 is the principles listing, including a malformed policy.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/cmd_rules/context.md`
