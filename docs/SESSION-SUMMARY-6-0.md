# spec-sync 6.0 — session summary

## Shipped

| PR | |
|---|---|
| **#499** merged | Deleted 7,257 lines of CI that reimplemented the SDD lifecycle against Git commit topology, plus two defects living in it. Closed #496/#497/#498 by deletion. |
| **#500** merged | Three lifecycle fixes. First PR in this repo to pass CI green with no bypass. |
| **#503** open | Six findings fixed. 10 commits, rebased on current main, CHG-0081 approved and verified. |
| **#28** open (sandbox) | 13 passing 6.0 drills, three-outcome runner, two lanes, five legacy drills guarded. |

Also merged by you: **#501**, **#502** — re-verified and archived CHG-0079/0080, which removed the compounding refresh cost.

## Findings: nine found, six fixed

| # | Finding | Status |
|---|---|---|
| 1 | `init` writes empty `verification_commands`; a fresh project cannot complete its own lifecycle | **fixed** |
| 2 | `audit --strict` reports the sequence ledger as uncovered | **fixed** (by 6) |
| 5 | `reopen`, `correct`, `correct-owner` unreachable — `finalize` never stops at `Accepted` | **fixed** (3 coupled changes) |
| 6 | `change new` fails on any branch missing an earlier change's directory | **fixed** |
| — | Owner correction rejected an archive-origin reopen | **fixed** |
| 8 | `supersede --digest` had no discoverable source | **fixed** |
| 3 | The Rust suite cannot see this class of defect | structural |
| 4 | The guided path produces a commit its own gate correctly refuses | **open — needs a decision** |
| 7 | `specsync new` scaffolds a spec `finalize` rejects | **open — two wrong fixes documented** |
| 9 | Can a `change depend` dependency ever be satisfied? | **open question** |

## Confirmed working

Worth stating after a day of cataloguing rough edges — these all hold:

- Separation of duties: self-review refused, independent review accepted
- `depend` ordering enforced while the dependency is draft or implementing
- `supersede` rejects a wrong predecessor digest
- A hand-resolved `change-sequence.json` merge conflict is **rejected, not trusted**
- The spec gate caught an undocumented public export I added, naming file and symbol

The core guarantees are sound. The friction is around them.

## The three lessons that cost the most

**1. The Rust suite cannot judge this work.** It passed 2,181 + 333 through all nine defects. Two of my own changes would have shipped green while quietly loosening a rule — the wholesale stub filter, and a `change-sequence.json` allowlist skip that changed the caller so the test asserting the function stayed green. Fixtures are single-process and single-root, so concurrency and multi-clone defects cannot fail there by construction. The effective-contract gate has had zero coverage since 5.0.

**Use the drills and the requirements as the judge.**

**2. Read the requirement before writing code.** Nine wrong calls. The cheapest catches all came from reading `REQ-change-*` first — `REQ-change-016` stopped a bad allowlist change, `REQ-change-033` settled the owner-correction gate correctly. The expensive ones came from writing first and discovering intent afterwards.

**3. Never swallow command output in a probe.** `>/dev/null 2>&1` turned fixture mistakes into apparent product defects three times. Drills have assertions to catch what you missed; ad-hoc probes do not.

Two mechanical traps specific to this repo:

- **`touch` before `cargo build`.** Script edits do not reliably trip cargo's fingerprint in a worktree; `cargo build` reports `Finished` and you test a stale binary. This produced one entirely false conclusion and looked like edits "vanishing".
- **Grepping an error message is unreliable.** Several are assembled — e.g. `format!("failed to read {} change state", if archived { .. })` — so the literal string never appears in source.

## The pattern that made this safe

Every fix followed: **drill asserts the bug → fix it → drill goes red to announce the fix → invert it to guard the fix.** That happened four times. It is what let six findings land in a day on a codebase whose test suite is blind to the whole class.

## Open for decision

- **Finding 4** — the gate is correct; the question is *when* `approvals.json` and the sequence ledger are written relative to the verification anchor.
- **Finding 7** — gate-side only. The scaffold-side option is ruled out: `SCAFFOLD_BOILERPLATE_PREFIXES` contains the scaffold's exact strings *by design*, and that coupling is the unfilled-section detector. Two gate-side attempts also failed; write the negative test first — an authored `## MODIFIED` section with a stub body must fail.
- **Finding 9** — blocked behind 7, since fixtures trip the scaffold gate.
- **`SPECSYNC_CANDIDATE_SHA`** — needed as a repo variable for PR #28's candidate lane.
- **SDD tool comparison** — requested, not done. Needs real research, not an aside.

## Note: lessons and learnings are not a first-class artifact

`ArtifactKind` is `Requirements | Research | Design | Plan | Tasks | Context | Testing | Docs | Custom(String)`.
There is no lessons, learnings, retrospective or postmortem artifact, and no CLI surface for one.

`Custom(String)` means `specsync change new --artifact lessons` creates `lessons.md` as a selected
artifact that must be completed before approval — so the mechanism exists, but nothing suggests it,
scaffolds it, or aggregates across changes.

Worth considering: this session produced nine findings and three durable lessons, none of which the
lifecycle had anywhere to record. They live in hand-written docs instead. A `lessons` artifact —
adaptive for `bug_fix` and `migration` kinds, aggregated by `change list` or a `lessons` command —
would let the tool accumulate what its users learn, which is exactly the material that made today's
work possible.
