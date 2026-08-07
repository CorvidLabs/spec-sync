# Three findings, 2026-08-03

Written up plainly. No code changes proposed here.

---

## 1. A fresh 6.0 project cannot run its own lifecycle

`specsync init` writes `"verification_commands": []`. Every lifecycle command then fails
on that empty list.

```
specsync init
specsync change new "..."          # works
specsync change check CHG-0001-... # fails
```

```
error: no verification commands are configured for this change;
       add a component command or a bounded project fallback in .specsync/sdd.json
```

Source: `src/change.rs:2523-2528`.

Reproduced using only 6.0 commands — no 5.x verbs involved. The user has to hand-edit
`.specsync/sdd.json` before the tool works at all. `change adopt` detects a test command
per `MIGRATION.md`, so the intended path likely runs through adopt, but plain `init`
leaves people stuck with an error that names a file they have never opened.

**Severity:** out-of-box blocker. First thing a new user hits.

---

## 2. Five sandbox drills are out of date, not broken behaviour

These five fail on 6.0 and pass on released 5.2.0:

```
008-squash-archive-regression   011-registry-stub-tolerance   013-batch-correct-owner
009-migrate-5-0-backfill        012-registry-parser-realities
```

They call `change start`, `verify`, `accept`, `archive` — verbs 6.0 removed deliberately
and replaced with `review`, `finalize`, and a repurposed `check`. The CHANGELOG documents
this as "One guided change workflow for SpecSync 6.0".

The split is exact: the drills calling removed verbs are precisely these five and no
others. The `026-*` and `022` drills carry `SKIP:` version guards; these five never got
one.

**This is not evidence of a 6.0 defect.** It was initially reported as five regressions;
that reading was wrong. They are un-migrated fixtures.

**Action:** rewrite them onto 6.0 verbs, or add the version guard the `026-*` drills use.
Until then every sandbox run is red for an uninteresting reason, which hides real signal.

---

## 3. The test suite cannot see the bugs that matter

The Rust suite is green at 2,181 unit + 333 integration. Both tiers use a single process
and a single `TempDir` repository root.

The lifecycle lock is `flock` on `.specsync/change.lock` — a per-root path. It cannot
serialize two roots. So:

| Test | Shape | Result |
|---|---|---|
| `concurrent_change_creation_assigns_unique_ids` (`src/change.rs:21288`) | 8 threads, **one root** | permanently green |
| `drills/026-multi-clone-new.sh` | 2 processes, **two roots** | reproduces a real ID collision |

Both are "concurrency tests". Only one can fail. The unit fixture is structurally
incapable of catching the bug.

The 6.0 integration tier contains zero occurrences of `concurrent`, `squash`, `rebase`,
`clone`, or `push`.

Related: the effective-contract validation gate has had **no test coverage in either
direction** since the original 5.0 commit. A change to its severity semantics passed the
entire suite while opening a path that silently empties a required spec section. Green
told us nothing.

**Consequence:** green tests are not evidence that 6.0 is ready. Behavioural drills against
a real binary, across multiple processes and repository roots, are where the remaining
defects live.

---

## 4. The guided path produces a commit its own gate refuses

Reproduced by following the workflow SpecSync itself prescribes:

```
change approve → change check → git commit → change review
```

```
error: scoped review cannot bind stale verification
       (verification descendant changed disallowed path
        `.specsync/changes/CHG-0001-p/approvals.json`)
```

Also fires for `.specsync/change-sequence.json`, and for the same reason.

### The gate is correct. Do not widen it.

`REQ-change-016` (`specs/change/requirements.md:185`) restricts what a commit between
verification and review may touch:

> Only `state.json`, `verification.json`, `verification-attempts.json`, `review.json`, and
> `review-attempts.json` …

Its purpose is stated in the same requirement: reject *"unintegrated, altered, or historically
tainted evidence"*. `approvals.json` carries the **scope-approval digest** — an authorisation,
not an output. Allowing it to move inside that window would let a scope be approved that
differs from the one verified. `change-sequence.json` is excluded by name in the same list.

The test `verification_persistence_allowlist_is_exact_and_canonical` (`src/change.rs` ~:28160)
asserts both rejections explicitly, citing REQ-change-013 and REQ-change-016.

**An earlier attempt to skip `change-sequence.json` at the call site was reverted.** It would
have passed the full suite: the test asserts the *function's* behaviour, and the change was in
the caller, so the assertion stayed green while the effective policy loosened. Any fix here must
be judged by the requirement, not by the suite.

### So the defect is the workflow, not the rule

`approve` and `change new` write files that the verification anchor then forbids moving. The
lifecycle instructs the author to make a commit its own gate refuses.

Candidate directions, none evaluated:

- **Set the verification anchor after those writes**, so the files predate the window rather
  than falling inside it
- **Have `change check` absorb the commit boundary**, so authors are not told to commit between
  verification and review
- **Separate authorisation from evidence on disk**, so `approvals.json` is not in the same
  directory the anchor policies

This is a design question about *when* things are written, not about what the gate accepts.
Decide the direction before writing code.

**Severity:** blocks the documented path for any change committed before review — the normal
order. Workaround today: do not commit between `check` and `review`.

**Reproduction:** `spec-sync-sandbox/drills/030-correct-owner-6.sh` avoids it by not committing
in that window, and says so in a comment.

## 5. Three recovery commands cannot be reached on 6.0

`change reopen`, `change correct`, and `change correct-owner` all appear in
`change --help`. None can succeed through the guided path.

`finalize_change` (`src/change.rs:5549-5556`) performs both halves in one command:

```rust
accept_change_with_gate(...)      // state = Accepted
archive_change_with_options(...)  // state = Archived
```

So `Accepted` exists only transiently inside `finalize` and is never a state a user can
stop at. There is no `accept` verb on 6.0 — the 5.x code path survives but is fenced off:

```
CHG-… uses the single 6.0 workflow; record scoped review and run `specsync change finalize`
```

Three functions require `Accepted`:

| Function | Requires | CLI verb |
|---|---|---|
| `reopen_change` (`:2568`) | `[Accepted]` | `change reopen` |
| `correct_interview_metadata` (`:4207`) | `[Accepted]` | `change correct` |
| `archive_change` (`:5587`) | `[Accepted]` | internal |

`correct-owner` requires `Verifying`, reachable only via `reopen`, so it is blocked one
step further back. Nothing transitions `Archived` back to `Accepted`.

Reproduced in `spec-sync-sandbox/drills/030-correct-owner-6.sh`:

```
error: cannot reopen accepted evidence while CHG-0001 is archived; expected accepted
error: cannot discover missing acceptance input owners while CHG-0001 is archived; expected verifying
```

**Why it matters:** these are the recovery tools. `reopen` exists to recover when accepted
evidence goes stale — a spec drops a file, ownership breaks, work needs re-verifying. On
5.x that was reachable. On 6.0 there appears to be no route.

**DECIDED 2026-08-03 by 0xLeif: reopen from archived.** `reopen` will operate on
`Archived` changes, restoring them to a working state. The alternative — having `finalize`
stop at `Accepted` with archiving as a separate step — was not taken; it would have
reintroduced a second step to the guided path that 6.0 deliberately collapsed.

### Scope — larger than relaxing the state check

Allowing `[Archived]` in `require_state` alone is **wrong** and would corrupt the workspace.
`reopen_change` writes to `change_dir(root, id)`, the *active* directory:

```rust
change_dir(root, &record.id).join("approvals.json")   // src/change.rs ~:2670
change_dir(root, &record.id).join("state.json")
change_dir(root, &record.id).join("change.md")
```

An archived change lives under `.specsync/archive/changes/`. Writing there would leave two
directories claiming the same change ID, in disagreeing states.

The correct shape is **un-archive, then reopen**: move the change directory back from
`archive/changes/` to `changes/`, set state to `Verifying`, and record the reopen in the
append-only ledger as today.

Already solved: `find_change_dir` (`src/change.rs:15192`) searches the archive, which is why
the failure is `is archived; expected accepted` rather than "not found". Loading works; only
the write path and the state gate need changing.

### Open sub-question

Should un-archiving be recorded as a **distinct audited event**, separate from the reopen
itself? An archived change that quietly becomes active again is a gap in the audit trail.
The existing `reopenings[]` ledger may or may not be the right home for it. Decide before
implementing — it is easier to add an event than to backfill one.

### Before starting

- Entry point: `reopen_change`, `src/change.rs:2568`
- Reproduction: `spec-sync-sandbox/drills/030-correct-owner-6.sh` asserts today's failure and
  is written to go **red** when this is fixed; invert those two assertions and extend the
  drill to cover `correct-owner` properly
- Watch the `archive_*` and `*_squash_*` test families — this touches archive-integrity, whose
  whole job is noticing an archived directory that moved

**Confidence:** four independent code sites agree and a drill reproduces it. Not proven
exhaustively — `adopt` or a hand-edited `state.json` might reach `Accepted`, though
hand-editing is not a real answer.

---

## 6. `change new` fails on any branch missing an earlier change's directory

The most ordinary git workflow there is:

```
git checkout -b dev-a
specsync change new "Feature A"     # creates CHG-0001-feature-a
git commit -am "change A"

git checkout main && git checkout -b dev-b
specsync change new "Feature B"
```

```
error: failed to read active change state
       .specsync/changes/CHG-0001-feature-a/state.json: No such file or directory (os error 2)
```

`.specsync/change-sequence.json` is committed and records `CHG-0001-feature-a` as active. The
change *directory* exists only on `dev-a`. On any branch without that directory, SpecSync
reads the ledger, tries to load a change that is not there, and fails hard.

**You cannot start a second change from a branch that does not contain the first one.** The
error names a missing file rather than saying the ledger and the working tree disagree.

Reproduced with 6.0 verbs only, in a fresh repository, in six commands.

### This is the third symptom of one design problem

| Finding | Symptom |
|---|---|
| 2 (coverage) | `audit --strict` reports the ledger as an uncovered path right after `change new` |
| 4 (review gate) | the review gate rejects a commit containing the ledger |
| **6** | `change new` fails outright on a branch missing an earlier change's directory |

All three trace to `.specsync/change-sequence.json` being **committed state that is
branch-sensitive**, while the changes it references live in directories that are also
branch-sensitive — and nothing keeps the two in step.

Worth treating as one design question rather than three bugs. Candidate directions, none
evaluated:

- Make ledger entries tolerate a missing directory (treat as not-active-here rather than an error)
- Keep the ledger out of version control and derive the sequence from directories present
- Reconcile ledger against directories on load, warning rather than failing

**Severity:** blocks `change new`, the first command in the workflow, in a completely
ordinary branching pattern. Arguably the most user-visible finding so far.

**Reproduction:** `spec-sync-sandbox/drills/031-specsync-merge-conflict.sh`, which hit this
during setup while trying to test something else.

---

## 7. `specsync new` scaffolds a spec that `change finalize` refuses

```
specsync new lib                      # tool generates specs/lib/lib.spec.md
specsync change new … --spec lib      # complete the guided path
specsync change finalize CHG-0001-…
```

```
error: effective contract `lib`: Section ## Dependencies contains only unfinished draft text
```

`specsync new` emits a scaffold whose `## Dependencies` and `## Purpose` sections carry
placeholder prose. Effective-contract validation promotes every validator warning to a hard
error (`src/change.rs`, `.chain(result.warnings)` in `validate_effective_contracts`), so the
stub warning becomes fatal at finalize.

**The tool authored the text it then rejects.** The user never wrote it.

**Reproduction:** `spec-sync-sandbox/drills/032-next-action-loop.sh`, which drives the guided
path using only `next_action` and sticks here.

### Bearing on decision A

Decision A was framed as "6.0 tightened draft-text tolerance and broke migration". That framing
was **wrong** — an adversarial audit showed migration never calls this gate, and the proposed
fix (filtering `StubSection` warnings wholesale) would have opened a silent content-loss path:
a `## MODIFIED` delta with an empty body writes an emptied section into the canonical spec, and
the stub warning is the only gate catching it. That change was reverted.

This finding is the **genuine narrow case** the audit described: a change blocked on stub text
in a canonical spec it did not author. The fix the audit recommended still stands:

- scope any exemption to sections **this change did not author** (the applied `SpecSection`
  delta keys name exactly which ones it did), plus the `canonical_applied && Verifying` case
  where no delta is replayed at all
- route it through `IgnoreRules` so `.specsyncignore` and inline directives finally work at
  this gate, and every suppression is reported rather than silent

### The scaffold-side fix is wrong — do not take it

An earlier draft of this finding suggested having `specsync new` emit scaffolds that pass the
gate. That would defeat the detector. `SCAFFOLD_BOILERPLATE_PREFIXES` (`src/parser.rs:838`)
contains the scaffold's exact strings:

```rust
"document this module's responsibility",               // scaffold's ## Purpose
"list runtime dependencies and the specific symbols",  // scaffold's ## Dependencies
```

The generator and the detector are **deliberately coupled** — that coupling is how "you have
not filled this in yet" is recognised at all. Rewording the scaffold to slip past its own
detector would silently disable unfilled-section detection for every project.

So the scaffold is *correctly* flagged. The defect is only that a correct warning is promoted
to a hard error at the finalize gate, for a section the change did not author. The gate-side
fix above is the whole fix.

**Severity:** blocks finalize for the ordinary first change against a freshly scaffolded module.
Combined with finding 1, a brand-new project cannot complete a change without editing two files
the tool generated.

---

## 8. `supersede` requires a digest with no way to obtain it

```
specsync change supersede --path <PATH> --spec <MODULE> --digest <DIGEST> <ID> <PREDECESSOR>
```

`--digest` is documented only as *"Full `specsync.acceptance-entry.v1` predecessor digest"*.

The value exists at `.specsync/changes/<predecessor>/verification.json` →
`acceptance_manifest.entries[].entry_digest`, matched by `path`. Nothing tells the user that:

- No CLI command emits it. `change show --json` exposes `summary.definition_digest` and
  `effective_definition.view_digest`; neither is an acceptance-entry digest.
- The help text names the format but not the source.
- `acceptance-entry` appears nowhere in `docs/` or `README.md`.

So the only route is: know the manifest exists, open an internal evidence file, find the entry
matching your path, and copy the field by hand.

**Why it matters:** `supersede` exists to adopt a predecessor obligation, which is a recovery
operation reached when something has already gone wrong. Requiring an undiscoverable argument at
that moment is the worst place for it. An agent driving from `--help` cannot construct the
command at all.

**Candidate fixes, none evaluated:**

- Expose entry digests in `change show --json` for accepted changes
- Add `--path`-based lookup so `supersede` resolves the digest itself, keeping `--digest` as an
  explicit override
- At minimum, name the source file and field in the help text

**Severity:** not a correctness defect. Squarely in the "hard to use, wastes time" class — the
kind of thing that makes a recovery path unusable in practice even though it works.

**Status:** no drill yet. `030` covers `correct-owner`; `supersede` has no coverage on 6.0.

---

## 9. OPEN QUESTION: can a `change depend` dependency ever be satisfied?

Not confirmed — recorded so it is not lost.

`change depend <ID> <ON>` declares ordering. `change check` on the dependent refuses until the
dependency is **accepted**:

```
error: dependency `CHG-0001-first` is implementing;
       it must be accepted before CHG-0002-second can start
```

But `finalize` performs accept and archive in one command, so a change is never observably
`Accepted` — that is finding 5. If the dependency check requires exactly `Accepted`, the same
way `reopen` and owner correction did, then **a declared dependency may never become
satisfiable through the guided path**.

Two possibilities, untested:

- The check accepts `Archived` as well, in which case this is fine
- It requires exactly `Accepted`, in which case `depend` is unusable for the same reason three
  recovery commands were

**How to settle it:** drive a dependency to `finalize`, then run `change check` on the
dependent. A probe attempt got as far as declaring the dependency and confirming the ordering
is enforced while the dependency is draft/implementing, but the dependency could not be
finalized because the fixture tripped finding 7 (scaffold `## Public API` prose) and a missing
`files:` frontmatter entry.

**Why it is plausible:** finding 5 turned out to be three coupled sites, each requiring
`Accepted` — `reopen`, `correct`, and owner correction. Dependency satisfaction is a fourth
place with the same shape. Two were found only by exercising the path.

**Confirmed so far:** `change depend` accepts the ordering, and `change check` correctly
enforces it while the dependency is draft or implementing. The ordering mechanism works; only
its terminal condition is in doubt.

---

## 10. DESIGN: archival should fold lessons into the spec's context

Not a defect — a missing lifecycle stage, specified here from the author's intent.

`finalize` moves a directory. In the intended model it is the moment knowledge moves from the
change into the spec, and it is the only place the system compounds rather than merely records.

### The three stages

| Stage | |
|---|---|
| **Proposal** | spec, design, requirements, planning and review agents, cleared context — reviewed and signed off. PR or local sign-off, either works. Approval is its terminal act. |
| **Implementation** | merged and *live* in `.specsync/changes/`, deliberately visible so other agents and reviews can see what is in flight and what was intended. |
| **Archival** | archive the change **and** fold lessons into the spec's companion context. |

Active changes on main are therefore intentional, not accumulation to be eliminated. The defect
was that they went stale and cost a re-verification per PR, which finding 5's fix addressed.

### Where lessons live

In `specs/<module>/context.md` — the **spec's** companion, not the change's. A per-change
lessons file dies with the change; the point is that a module accumulates what was learned
about it across every change that touched it.

Written by **the agent**, at archival, drawn from the change's commits, PR, and PR comments —
synthesised from what actually happened, not recalled.

### How, without spec-sync becoming opinionated

SpecSync must not shell out to a particular agent. It does not need to: the agent driving the
lifecycle just ran `finalize`. So `finalize` assembles the material and sets `next_action`:

```
Next: write lessons for `change` into specs/change/context.md
      from .specsync/changes/CHG-XXXX/lesson-bundle.md
```

Same mechanism the lifecycle already uses everywhere, and drill `032-next-action-loop.sh`
confirms agents can follow `next_action` to termination. Neither blocking nor nagging — a step.

### Two pieces of work

1. **Capture** — the change accumulates its commit range, PR number and PR comments as it goes.
   Some is already reachable (`verification.json` records commits, review evidence records
   verdicts); PR comments need fetching. Mechanical.
2. **Synthesise** — `finalize` assembles the bundle, sets `next_action`, and completes archival
   once the spec's context has been written.

### On unbounded growth

Raised and answered: if a module's lessons grow without bound, that is a signal the module is
too large or too hot, and compaction would hide exactly what you want to see. Treat volume as a
health signal. `specsync compact` exists if it is ever needed, but routine compaction is not the
goal.

There is likely a second tier for lessons that are project-wide rather than module-specific.
This session's own output splits that way:

- *module-specific* — the effective-contract gate has had no coverage since 5.0; `finalize`
  collapses accept and archive so `Accepted` is never observable
- *project-wide* — `touch` before `cargo build` or you test a stale binary; grepping an error
  message is unreliable because several are assembled with `format!`; the test fixtures are
  single-process and single-root, so a whole class of defect cannot fail there

Leave the project-level tier undesigned until several modules have accumulated something real.

### Status

`feat(change): prompt the context artifact` was initially written as lesson prompts on the
change's context artifact. That was the wrong home and has been rescoped: the change's context
now prompts for its own working record — what led here, what a session picking it up mid-flight
needs to know. Lessons await this archival stage.

---

## Also observed (drill 030)

- 6.0 refuses approval until every selected artifact carries real content **and** a
  semantic delta exists per affected spec. 5.x approved without either. This is a real
  workflow difference agents must be told about; nothing in the docs states it as a
  precondition of `approve`.

## Also observed (earlier)

- `drills/027-ship-sequence.sh` greps `--help` output and checks two docs exist. It never
  runs a lifecycle, so the ship/finalize path has no behavioural coverage in the sandbox.
- `examples/sdd-lifecycle/run.sh`, `examples/sdd-concurrent-changes/run.sh`, and
  `examples/sdd-five-epics/run.sh` are real-binary harnesses referenced by no CI job or
  test. `sdd-concurrent-changes` is sequential with one actor despite the name.
- Squash merges leave a change's recorded verification commit unreachable from main, so
  squash-merged changes cannot be finalized. Both changes currently on main are
  unfinalized for this reason.
