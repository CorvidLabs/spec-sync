---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: research
---

# Research

Done on 2026-11-11. The binaries were the v6.2.0 release asset `specsync-macos-aarch64.tar.gz` (sha256 matched its sidecar, and it is byte-identical to Homebrew's 6.2.0) and a build of main at 4eed6e8d. Each scratch repository is a workflow-v2 project with a bare origin. A change got a relayed approval from main's binary, and its `state.json` (and, once archived, `accepted-state.json` too) was then raised to `workflow_version: 3` by hand. Every 6.2.0 run below left `.specsync/` byte-identical.

| Repository holds | 6.2.0 command | Result |
|---|---|---|
| active v3 change A | `change status A`, `change approve A` | exit 1, "was written by a newer SpecSync (workflow version 3); upgrade specsync to read it" |
| active v3 change A | `change approve B` (unrelated draft) | exit 1, same message for A |
| active v3 change A | `change list`, `change audit` | exit 1, same message |
| active v3 change A | `change status B`, `check` | exit 0 |
| archived v3 change A | `change new`, `change audit`, `change status A` | exit 1, same message for the archived `state.json` |
| archived v3 change A | `check --force`, `check --strict --require-coverage 100 --force`, `lifecycle enforce --all`, `comment`, `change list` | exit 0 |

Main's binary also refuses the hand-raised record, as expected: it knows only versions 1 and 2.

The code on main:

- `validate_loaded_change` and `validate_workflow_version_anchor` accept (1, none or 1) and (2, 2).
- `validate_workflow_version_history` refuses any version or origin change between commits, and between `HEAD` and the working tree.
- Only the execution digest (`definition_digest_for_correction_count`) serializes the whole `ChangeRecord`, and with it `workflow_version`. The v2 scope digest, the workspace digest (`.specsync/changes/` is excluded), and the closing, review and finalization digests do not.
- `change review` writes only `review.json` and `review-attempts.json` today.

What the Action and Trust run: the Action runs `specsync check --force`, `lifecycle enforce --all` and `comment`. Trust v1.2.1 (`dd52a7a9`) runs only `specsync check`. This repository's CI runs `change audit` with the branch build.
