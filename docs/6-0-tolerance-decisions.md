# Three tolerance decisions blocking 6.0

> **DECIDED 2026-08-03 by 0xLeif.** A: scope to authored specs. B: exempt no-op changes.
> C: exempt metadata-only corrections. Rationale for each is the recommended option below.

Five sandbox drills that pass on released 5.2.0 fail on 6.0. They collapse to three
places where 6.0 got stricter, each with a legitimate counter-example the stricter rule
did not anticipate.

None looks like a correctness break. All three are product calls about what the tool
should tolerate — which is why they are written up rather than fixed.

Reproduce any of these with:

```
SS=<6.0 binary> SPECSYNC=<6.0 binary> bash drills/<name>.sh    # in spec-sync-sandbox
```

---

## A — Should an effective contract accept scaffolded sections?

**Breaks:** `008-squash-archive-regression`, `009-migrate-5-0-backfill`,
`012-registry-parser-realities` — three of the five.

```
error: effective contract `lib`: Section ## Purpose contains only unfinished draft text;
       effective contract `lib`: Section ## Dependencies contains only unfinished draft text
```

5.2.0 accepted these sections. 6.0 rejects them.

**Why it matters:** the paths that break are *migration* and *registry parsing* — the two
places whose job is to handle content that has not been written yet. `009` backfills a
5.0-era ledger; a backfilled spec is scaffolding by definition. Requiring finished prose
before a migration can complete means a 5.x project cannot reach 6.0 without hand-editing
every generated section first.

**Options**

1. **Scope the rule to authored specs.** — **CHOSEN** Migration and scaffolding paths tolerate draft
   sections; specs a human is actively delivering do not. Keeps the guarantee where it has
   value, removes it where it is self-defeating. *Recommended.*
2. **Warn instead of erroring** in effective-contract evaluation. Simpler, but weakens the
   guarantee everywhere including where it was working.
3. **Keep as-is** and change migration to emit finished placeholder prose. Honest, but
   generates text nobody wrote and everybody must then rewrite.

**Note:** this is the mirror image of #495. There, `# TODO` headings pass scope approval;
here, draft sections fail contract evaluation. Same subsystem, opposite direction — worth
deciding together so the two rules agree on what "unfinished" means.

---

## B — Should an inert stub complete a lifecycle?

**Breaks:** `011-registry-stub-tolerance`

```
inert stub failed at accept: error: CHG-0001-registry-probe uses the single 6.0 workflow;
record scoped review and run `specsync change finalize CHG-0001-registry-probe`
```

Not a defect — workflow-v2 asserting itself. But the drill's subject is an *inert registry
stub*: a probe that changes nothing. Requiring a scoped review and a finalize for it is
ceremony with no reviewable content.

**Options**

1. **Exempt no-op changes** — **CHOSEN** — from scoped review and finalization when the delta is empty.
   Needs a precise definition of "inert" — likely: no canonical spec delta and no tracked
   file change outside `.specsync/`.
2. **Keep the requirement** and change the drill to complete the lifecycle. Consistent, but
   concedes that trivial changes cost a full lifecycle.
3. **Scope by change kind** — `operations`/`documentation` kinds skip review. Blunter, and
   invites mislabelling to dodge review.

Interacts with the open scoped-review question: if a review binds to implementation content,
an inert change has no content to review, which argues for option 1.

---

## C — Should metadata correction require verification commands?

**Breaks:** `013-batch-correct-owner`

```
error: no verification commands are configured for this change;
add a component command or a bounded project fallback in .specsync/sdd.json
```

`change correct-owner` rewrites accepted-evidence ownership. It changes no code, so there is
nothing for a verification command to verify — but 6.0 demands one be configured.

**Options**

1. **Exempt metadata-only corrections** — **CHOSEN** — from the verification-command requirement. The
   audit trail is append-only regardless, so the evidence guarantee is unaffected.
   *Recommended.*
2. **Allow an explicit empty command set** for correction operations, recorded as such.
   More auditable, more ceremony.
3. **Keep as-is.** Every project must configure a command purely to satisfy a code path that
   cannot use it.

---

## Cross-cutting

All three share a shape: **6.0 tightened a rule without the case that needed tolerating.**
The Rust suite is green at 2,181 + 333 — these paths are exercised from the outside, by a
person or an agent using the tool, and nothing tested that until the sandbox was unpinned
from 5.2.0 today.

That is the argument for the scenario matrix (solo dev, multiple devs, async clones, merge
topologies, conflicts, green and bad paths) rather than fixing three bugs and calling 6.0
done.

**Also outstanding:** `027-ship-sequence` fails because it asserts `ship-status`/`ship`
exist. Those were deliberately dropped with #487 and #499 removed the tip dance that
motivated them. That drill needs updating or deleting — it is a drill bug, not a product
bug.
