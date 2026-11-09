# Lesson bundle: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Add specsync doctor to report every change problem with its fix and close merged changes late where older SpecSync never reads
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args, cli
- **Paths**: .github/scripts/test-closed-late-compat.sh, .github/scripts/test-required-ci-gate.py, .github/workflows/ci.yml, AGENTS.md, CHANGELOG.md, docs/HLD.md, fledge.toml, hi/recover.md, hi/ship.md, INTENT.md, site/src/content/docs/cli.md, site/src/content/docs/workflow.md, specs/change/, specs/cli/, specs/cli_args/, specs/cmd_change/, src/change.rs, src/change/doctor.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/doctor_collision.rs, src/cli.rs, src/commands/change.rs, src/commands/doctor.rs, src/main.rs, tests/integration.rs, tests/integration/doctor.rs
- **Acceptance**: RECOVER-3, RECOVER-3.a (REQ-cmd-change-035): `specsync doctor` reads the repository and lists every finding with its kind, its change, the reason and its fix, classed safe, decision or report-only. The kinds are: a change merged but never archived (an active package committed on the remote default branch); an accepted change whose inputs drifted, including any package in .specsync/archive/changes/ that still says accepted; a finalize that stopped halfway (accepted with finalization.json, or a v1 change accepted but not archived); stale verification; and records that another version of SpecSync wrote (5.0.1-era ledgers missing digest fields, v1 plans missing the 6.2 sections, a schema newer than the binary, a closed-late slug minted again).
- **Acceptance**: RECOVER-3.b: plain `doctor` writes nothing, and `doctor --fix --dry-run` reports exactly what `--fix` would do with the same arguments and writes nothing.
- **Acceptance**: RECOVER-3.c: `--fix` applies only the safe fixes: the `migrate 5.0` backfill, and resuming a stopped finalize (v1: `change archive`) when nothing blocks it. Stale verification is report-only and names `change check <id> --commit`. A late close is a decision: it applies only to changes named with `--close-late <id>...` or `--close-late all`, and only with a non-empty `--actor` and `--reason`.
- **Acceptance**: RECOVER-3.d: a check that cannot run, such as a bounded Git read that overflows (#976), becomes a finding that says so, and doctor still reports every other finding.
- **Acceptance**: SHIP-12, SHIP-12.c, SHIP-12.e (REQ-change-143): a late close moves the package byte for byte from .specsync/changes/ or .specsync/archive/changes/ to .specsync/archive/closed-late/<date>-<id>/, keeps its state.json and the state in it, adds late-close.json, and adds no ChangeState variant. It reads none of the change's delivery evidence, and it closes only a package committed on the remote default branch; any other is refused and nothing is written.
- **Acceptance**: SHIP-12.a, SHIP-12.d: late-close.json records the actor, reason, timestamp, the state it was closed from, the workflow version, the path it came from, the remote default ref and commit, the commit that merged it and that commit's date, and verified false. `change status` (text and JSON) and `ship-status` say it was closed late, never re-checked and not verified, and a closed draft also says it was never approved.
- **Acceptance**: SHIP-12.b: a closed-late change is never part of the archive preflight or the successor walk; reopen, review, check, accept, archive and finalize refuse it; and `change new` refuses its slug.
- **Acceptance**: SHIP-12.f: several late closes in one call are all checked before anything is written; one refusal writes nothing and names the change refused; a change named more than once is closed once; and each safe fix is revalidated against a fresh scan after every move, so a change closed late in the same call gets no safe fix.
- **Acceptance**: D5: late-closing a member of an acknowledged sequence collision takes the change out of that acknowledgement in .specsync/change-sequence.json, drops the acknowledgement once fewer than two members remain, and records the acknowledgement as it stood in late-close.json. The ledger sequence is never lowered, and doctor says the ledger edit needs an open change that declares it wherever meaningful files require one.
- **Acceptance**: SHIP-12.g, D4: a CI job and a fledge lane run real 6.0.0 and 6.2.0 Linux release binaries, pinned by sha256, against a repository after `doctor --fix`. There, audit passes; list omits the closed-late changes; status, ship-status and reopen report them not found; a partial finalize they blocked resumes and archives; a v1 archive passes; and a collision fixture audits clean.
- **Acceptance**: REQ-cli-args-025: the `doctor` subcommand parses `--fix`, `--dry-run`, a repeatable `--close-late <id|all>`, `--actor`, `--reason` and the global output flags, and argument parsing refuses `--close-late` given without `--fix`, `--actor` and `--reason`.
- **Acceptance**: Failing tests first: each new test fails on the branch base before the implementation and passes after; no existing gate, bound, timeout or assertion is loosened.

## Evidence

- Verification commit: `c0e905dd10d7c79ff19544047e1f41b614a5f83b`
- Base commit: `1276e314790521904552e375a659ae3214aa04dc`
- Verified by: `specsync check --spec change --spec cli --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

#833: archived changes left in state `accepted` have no supported repair, and one of them blocks every later
`change finalize`.

- **Corvidinho:** 46 such packages.
- **CorvidLabs/site:** 41 changes merged and never archived (the 38 to close plus 3 drafts). CHG-0026 blocks
  every finalize. Its documented recovery, `reopen`, overflows a bounded Git read (#976, fixed separately).
  The 15 v1 records cannot finish through `accept`. Site #616 sits in the partial-finalize state.

Leif's direction (relayed by orc t18, 2026-11-06) supersedes a standalone `close-late` command and a new
`closed_late` state. Build `specsync doctor`. It lists every problem with its fix, `--fix` applies the safe
ones, `--dry-run` shows them, and decisions need `--actor`/`--reason`. The late close is one doctor fix,
stored where SpecSync 6.x never looks. It ships in 6.3.0, and the site and Corvidinho CI pin 6.0.0.

Intent, captured verbatim with Leif's source:

- SHIP-12 and SHIP-12.a to SHIP-12.f ("Capture all + approve design");
- SHIP-12.g, RECOVER-3 and RECOVER-3.a to RECOVER-3.d ("Capture + recommended design").

Decisions D2 to D5 are recorded in the constraints answer. Requirement IDs were claimed by the orc:
REQ-change-143 (reworded), REQ-cmd-change-035 and REQ-cli-args-025.

The never-approved draft `close-a-change-that-merged-but-was-never-archived-…` is removed in this commit. Its
approval was refused with "semantic delta modules must exactly match affected specs".

Re-scope after the first review: Leif approved and reviewed this change at 2e65c8a8 and 45772452. `change ship`
then refused it with "acceptance input `src/agents.rs` is production source without deterministic canonical
ownership", because the approved affected paths named all of `src/`. Acceptance gives every production file in
scope an owner among the affected specs, and 127 of the 164 files under `src/` belong to other modules. Leif sent
it back (recorded by orc t18; option A). The affected paths now name exactly the files the change touches. `cli`
joins the affected specs because it owns `src/main.rs`, which gains the doctor dispatch arm, and its Purpose now
says that it routes `specsync doctor`. The code, tests and acceptance criteria are unchanged. The review the
send-back cancelled stays in the append-only history.

## From the change's design.md

# Design

**Command.** `specsync doctor [--fix] [--dry-run] [--close-late <id>|all ...] [--actor <human> --reason <text>] [--json]`

- `src/commands/doctor.rs` renders the command and is added to cmd_change's files.
- `src/change/doctor.rs` holds the scan and the fixes in the change domain.

**Scan.** It is read-only. Each finding records:

- kind and change ID;
- location (active, archived, closed-late);
- reason;
- fix, with its class: safe, decision or report-only;
- the exact command or doctor flag that applies it.

A check that fails to run becomes a finding with its error, for example the #976 overflow on an accepted
change's evidence. The scan continues.

| Kind | Detected by | Fix and class |
|---|---|---|
| merged, never archived | an active package whose `state.json` exists on `remote_default_ref` | decision: `--close-late <id>`; the normal finish is also named when the change is ready |
| accepted, inputs drifted | an active `accepted` change whose accepted-input validation fails, with its reason; an `accepted` package under `archive/changes/` | decision: `--close-late <id>`; report: `change reopen <id> --actor --reason` |
| stopped finalize | active, `accepted`, has `finalization.json` (v2); v1 accepted and not archived | safe: resume `change finalize` (v1: `change archive`) when no other finding blocks it; otherwise it names the blocker |
| stale verification | a verifying or implementing change whose verification is stale for the tree | report-only: `change check <id> --commit` (D2) |
| other-version records | 5.0.1-era ledgers missing digest fields; v1 plans missing the 6.2 sections; a schema newer than the binary; an ID in `changes/` or `archive/changes/` that a closed-late package already holds | safe: `migrate 5.0` for the backfill; report for the rest |

**Fix.**

- `--fix` applies the safe fixes, in a deterministic order.
- `--close-late` applies the decision for the named changes, or for every eligible one with `all` (D3).
- `--close-late` needs `--fix`, `--actor` and `--reason`.
- `--dry-run` with `--fix` prints the same plan and writes nothing.

**Late close (REQ-change-143).**

- **Move.** The package moves, byte for byte, with the durable rename `archive` uses, to
  `.specsync/archive/closed-late/<YYYY-MM-DD>-<id>/`. Its `state.json` keeps its state, and no
  `ChangeState` variant is added.
- **Merge proof.** The package's `state.json`, at its current path, must exist on the remote default. The
  record keeps the commit that added it (`git log -1 --diff-filter=A <ref> -- <state.json>`) and its date.
- **Reads not taken.** Delivery inputs, manifests, digests, verification and the preflight.
- **Collisions (D5).** For a member of an acknowledged sequence collision, the change is taken out of the
  acknowledgement in `change-sequence.json`; the acknowledgement goes once fewer than two members remain, so
  the members SpecSync still enumerates stay acknowledged. The acknowledgement as it stood is copied into
  `late-close.json`. The sequence and the high-water claim are untouched.
- **Safe fixes after a move.** Each safe fix is revalidated against a fresh scan after every late close or
  applied fix, and a change closed late in the same call gets none (Codex review of #994).
- **All or nothing.** Every named change is validated first. A write failure restores the packages already
  moved.
- **`late-close.json`.**
  - schema_version, change_id, actor, reason, timestamp
  - from_state, workflow_version, from_path
  - default_ref, default_commit, merged_commit, merged_at
  - verified: false
  - dropped_collision_acknowledgement (optional)

**6.3 reads of the closed-late folder.**

- **Roster and preflight.** Unchanged: they enumerate only `changes/` and `archive/changes/`, so a
  closed-late change is never preflighted, never a successor and never coverage.
- **Lookup.** `load_change` falls back to the closed-late folder only to report it.
  - `change status` and `ship-status` say: closed late on <date> by <actor> (<reason>); merged in <commit>
    on <date>; never re-checked, not verified. A draft adds: never approved.
  - JSON carries `closed_late: {...}` and `verified: false`.
- **Refusals.** reopen, review, check, accept, archive and finalize refuse it with that sentence.
- **New changes.** `change new` refuses its slug.
- **Duplicates.** Doctor reports an ID that a closed-late package and an `archive/changes/` or `changes/`
  package both hold.

**Compatibility test (D4).** `.github/scripts/test-closed-late-compat.sh`:

1. Download the 6.0.0 and 6.2.0 `specsync-linux-x86_64` release assets, pinned by sha256.
2. Build the fixtures with the PR binary and run `doctor --fix --close-late …`.
3. Check the old binaries' audit, list, status, ship-status, reopen, finalize-resume and v1 archive, plus a
   collision fixture.

It runs as a CI job on the Linux lane and as a fledge lane.

## From the change's testing.md

# Testing

Every new test fails on 1276e314 first, and that is recorded.

1. **Scan.** A fixture with one change of each finding kind gets each finding with its kind, reason, class and
   fix. A check that errors, simulated with an assume-unchanged delivery input, becomes a finding, and the rest
   are still reported (RECOVER-3, .a, .d).
2. **Read-only.** Plain `doctor` and `doctor --fix --dry-run` leave the tree byte for byte (RECOVER-3.b).
3. **Safe fixes.** `--fix` resumes a stopped v2 finalize and a v1 archive when nothing blocks them, and runs
   the `migrate 5.0` backfill. It leaves stale verification as a finding naming `change check <id> --commit`
   (RECOVER-3.c, D2).
4. **Late close.** `--fix --close-late <id> --actor --reason` moves a merged package byte for byte to
   `.specsync/archive/closed-late/<date>-<id>/` and writes `late-close.json` with every field. It reads no
   delivery input; a v1 change whose evidence read refuses still closes. A hand-archived `accepted` package
   moves out of `archive/changes/` (SHIP-12, .a, .c).
5. **Refusals.** Each writes nothing:
   - an unmerged change, including one in the partial-finalize state;
   - no remote default;
   - an empty `--actor` or `--reason`;
   - `--close-late` without `--fix`;
   - a change already closed late or archived.

   One refusal among several IDs writes nothing and names the refused change (SHIP-12.d, .e, .f).
6. **6.3 treats it as never verified.**
   - status (text and JSON) and ship-status report the late close, and a draft says never approved;
   - reopen, review, check, accept, archive and finalize refuse it;
   - `change new` refuses its slug;
   - a finalize it blocked before now passes, and #616's shape resumes (SHIP-12.a, .b).
7. **Collision.** Late-closing a member of an acknowledged collision drops and records the acknowledgement,
   leaves the sequence untouched, and says that the ledger edit needs a covering change (D5).
8. **Compatibility.** `.github/scripts/test-closed-late-compat.sh` runs the real 6.0.0 and 6.2.0 binaries
   after `doctor --fix`, with the assertions in the design (SHIP-12.g, D4).
9. **Grammar.** `--close-late` without `--fix`, `--actor` and `--reason` is refused by argument parsing
   (REQ-cli-args-025).

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-143 | tests 4, 5, 6, 7, 8 |
| REQ-cmd-change-035 | tests 1, 2, 3, 5 |
| REQ-cli-args-025 | test 9 |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| RECOVER-3, .a, .d | test 1 |
| RECOVER-3.b | test 2 |
| RECOVER-3.c, D2, D3 | tests 3, 5 |
| SHIP-12, .c, .e | tests 4, 5 |
| SHIP-12.a, .d | tests 4, 6 |
| SHIP-12.b | test 6 |
| SHIP-12.f | test 5 |
| D5 | test 7 |
| SHIP-12.g, D4 | test 8 (CI job) |
| REQ-cli-args-025 | test 9 |

## Verification commands

- `fledge lanes run verify` on the pinned rust 1.89
- `specsync change check <id> --commit`
- `specsync change audit --strict`
- `.github/scripts/test-closed-late-compat.sh`
- `fledge lanes run pre-push`

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
- `specs/cli/cli.3md`
