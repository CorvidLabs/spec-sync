# Lesson bundle: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Record a change that holds a relayed decision at workflow version 3 so released SpecSync 6.2.0 refuses it instead of dropping relayed_by
- **Kind**: BugFix
- **Specs**: change
- **Paths**: specs/change/, src/change/, src/change_tests/, src/change_tests.rs, tests/integration/relayed_decision.rs, .github/scripts/test-relay-gate-compat.sh, .github/workflows/ci.yml, fledge.toml, CHANGELOG.md, README.md, site/src/content/docs/workflow.md
- **Acceptance**: A change that holds a relayed approval, review or correction is recorded at workflow version 3 (origin 2, every digest unchanged), so the real SpecSync 6.2.0 release binary refuses it with 'was written by a newer SpecSync (workflow version 3); upgrade specsync to read it' and changes nothing, as REQ-change-158 states

## Evidence

- Verification commit: `e67449366f5bac7d190d6befbd06733940972160`
- Base commit: `4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#1064, a follow-up to #1043 and #1044. `relayed_by` (#633, #900) and `cancelled_review_attempts` (#995) landed on main after 6.2.0. Released 6.2.0 deserializes `approvals.json` and the review ledgers through typed structs that ignore unknown fields, then re-serializes them on every lifecycle rewrite. So the first time 6.2.0 writes to a change on which a newer binary recorded a relayed decision, it drops `relayed_by` and exits 0. #1044 made newer binaries keep unknown members, but 6.2.0 is already released. The only lever is something 6.2.0 already refuses: `load_change` rejects a `state.json` whose `workflow_version` is not 1 or 2, with "was written by a newer SpecSync (workflow version N); upgrade specsync to read it".

Leif chose option (a), his word "A", relayed by the coordinator agent t370 on 2026-11-11: 6.3.0 must stop released 6.2.0 from rewriting a change record that carries a relayed decision, rather than let it silently drop `relayed_by`. t370 checked that REQ-change-158's wording matches that choice before approval.

What a session picking this up needs to know:

- The gate covers the whole repository, not one change. I measured this with the real 6.2.0 binary on a scratch repository holding a hand-raised version-3 record (research.md). 6.2.0 lists every change, so one active version-3 change also makes it refuse `approve` on any other change, plus `change list` and `change audit`. One archived version-3 change makes it refuse `change new` and `change audit` for good. `check` (including `--strict --require-coverage 100 --force`), `lifecycle enforce --all` and `comment` still pass. So the GitHub Action and Trust v1.2.1, which runs only `specsync check`, keep working when pinned to 6.2.0. In plain terms: once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later. The PR body and the 6.3.0 release notes say so (t370's request).
- Chough 0.6.0 passes relays to `change review` from `chough change ship`, so CorvidLabs repositories will reach this state quickly.
- Workflow-v1 changes are not gated. The `--portable-5-0-1` pair requires v1 and exists so old binaries can read it; approve and review print a note. t370 chose this over refusing v1 relays (coordinator, 2026-11-11), which keeps REQ-change-relayed-decision-001 intact. The PR body and the release notes say: relays on legacy workflow-v1 (portable 5.0.1) changes are not gated, and approve and review say so.
- Not retroactive: 7 archived changes in this repository and 1 in CorvidLabs/site already hold relays at version 2. Archives are not edited.
- The issue's part (b), a latent array-length gap in `collect_unknown_members`, is out of scope.

## From the change's design.md

# Design

- **Version 3 is workflow v2 plus the gate.** `RELAY_GATED_WORKFLOW_VERSION = 3`. Code that branches on `workflow_version >= 2` or `< 2` already treats 3 as v2. The one literal `== 2` comparison (the archived same-PR finalization check) becomes `>= 2`. Loading accepts versions 1, 2 and 3. The anchor accepts (3, Some(2)) on active and historical reads, and refuses (3, None) and (3, Some(other)).
- **History.** `validate_workflow_version_transition` and the HEAD-versus-working-tree check accept one move: 2 to 3, with an origin of 2 (or an omitted origin on an old v2 record) becoming 2. Everything else is still refused, 3 back to 2 included.
- **Digests.** `definition_digest_for_correction_count` serializes the record with version 3 written as 2. Every recorded execution digest, and so verification, review and closing currency, is the same before and after the raise.
- **Raise points.**
  - `approve_definition_with_relay`: when a relay is given on a v2 change, set the version to 3 and save `state.json` before the approval is appended.
  - `record_scoped_review_with_relay`: on a v2 change, `state.json` goes first in the same prepared write as `review.json` and `review-attempts.json`.
  - A workflow-v1 change keeps version 1. A `--portable-5-0-1` pair needs v1, and exists so old binaries can read it. Approve and review print a note there: SpecSync 6.2.0 and older can still drop the relay. REQ-change-relayed-decision-001 is unchanged.
  - Every other `state.json` write (`save_change`, materialize, accept/finalize) goes through one helper. It raises a v2 record to 3 when the change directory's `approvals.json`, `review.json`, `review-attempts.json` or `corrections.json` holds a `relayed_by`. That covers corrections and records written by unreleased builds.
- **Archive.** Finalize copies the loaded record, so the archived `state.json` and `accepted-state.json` keep version 3.
- **Rejected:** refusing relays on workflow-v1 changes. It would remove the portable pair's relay (REQ-change-relayed-decision-001), and the pair exists for old binaries. Also rejected: creating every new change at version 3. It would lock 6.2.0 out of every repository 6.3.0 touches, relay or not. Also rejected: editing archived v2 changes that hold relays, because archives are not edited.

## From the change's testing.md

# Testing

| Check | Expected |
|---|---|
| Relayed approve on a v2 change | `state.json` is at version 3 with origin 2, and approval and digest are as before |
| Relayed review on a v2 change after `check --commit` | `state.json` at 3; the earlier verification still binds; `ship` archives with version 3 in `state.json` and `accepted-state.json` |
| Approve and review without a relay | version stays 2, bytes as today |
| Relay on a workflow-v1 change, including a `--portable-5-0-1` pair | recorded, version stays 1, and approve and review print the note that 6.2.0 and older can still drop it |
| History: 2 to 3 | accepted between commits and between `HEAD` and the working tree |
| History: 3 to 2, origin changed, (3, None) | refused |
| A rewrite of a v2 change whose ledgers hold a relay | raises it to 3 |
| Version 4 | still "written by a newer SpecSync" |
| `test-relay-gate-compat.sh` | real 6.2.0 (sha256-pinned) refuses the gated change's lifecycle commands with the newer-SpecSync message and leaves `.specsync/` byte-identical; after archive, `change new` and `change audit` refuse the same way and `check --strict` passes. Negative control: the same records without the raise let 6.2.0 rewrite them and drop `relayed_by` |

Locally: the targeted unit and integration tests, the compat script, fmt, and clippy on the touched code. The full suite runs in hosted CI.

## Where these lessons go

- `specs/change/change.3md`
