# Native lifecycle and evidence

Installed `specsync change status <id> --json`, `specsync change ship-status <id>` and
command help are authoritative for the current state and recovery path. This
reference follows SpecSync 6.1; do not hardcode it over another version's output.
Match the SpecSync version used by lifecycle writers and CI.

You run the native commands in this reference as part of Chough's current-agent
workflow. The human gives the goal and authorization and reviews the resulting PR;
they do not need to dispatch SpecSync or Fledge operations themselves. Report the
result and genuine decisions still needed, then perform authorized recovery steps.

## Shape of a change

```text
new → interview → artifacts and delta → one human scope approval → implement →
check (when requirements change) → cases run → gate → commit with .chough/ →
check --commit → push, CI → human implementation review → review → ship/finalize →
clear gate, stage advance → push archive, CI → merge
```

Between `check --commit` and finalize, nothing writes to `.chough/` and nothing is
committed: SpecSync's verification digest covers `.chough/`. SpecSync 6.1.2 ships an
opt-in exclusion, `verification_ignored_paths`, but Chough's own repository has not
adopted it, because entries are directory prefixes and covering `.chough/session.json`
would also hide `.chough/cases/`. Unless a project lists only safe directories, any
Chough write in that window forces another `change check`. See [accept.md](accept.md)
for the full order.

| Native state | Chough stage | Gate |
|---|---|---|
| draft | intent, design, plan | build, test, accept and done refused |
| approved, implementing, verifying, with `approval_valid` true | build, test, accept | build, test and accept allowed; done refused |
| archived, with authenticated `terminal_evidence` | test, accept, done | done allowed; build refused (new work needs a new change) |

Chough reads `summary.state`, `summary.approval_valid`, `summary.correction_valid` and,
for archived changes, `summary.terminal_evidence.validity`. The same prerequisite
applies whichever command moves the stage (advance, skip, back, reset, return or
release → done). It refuses when status is missing, cannot be decoded or reports an
invalid approval or a correction reported invalid (`correction_valid` false), when no
native change is recorded in the checkpoint (`refused:no-native-change`; record one
with `chough checkpoint --native-change <id>`), and when `.specsync/sdd.json` is not a
valid SpecSync policy. Two cases are not gated at all: with no SpecSync configuration
the gate reads `ungated:not-configured`, and with no `.specsync/sdd.json` or one with
`"enabled": false` it reads `ungated:sdd-disabled`. It never runs `approve`, `review`
or `finalize` itself. The current agent invokes those native writers when the
applicable human authority exists.

## Approval

Before recording `specsync change approve <id> --actor "<agent-account>"`, identify the
actual human authority that covers this concrete definition. It may be an explicit
prior decision or a standing instruction authorizing a bounded class of changes,
such as repository ticket implementations delivered through PRs. Record the source,
covered scope and limits in the checkpoint decisions and native context, and cite
that source in the native approval note. Record the acting agent account as the
native actor, separately from the human authorization source, then continue without
asking the human to repeat permission for this internal operation. Do not use the
human's name merely because they authorized the agent's work.

If the proposed work exceeds that authority, ask the consequential scope question.
An ordinary keep-working instruction without applicable authority, an agent label,
a bot review or a passing check does not create human approval. A changed approved
definition follows native reapproval rules; existing standing authority applies
only while it still covers the new scope. Actor text is a portable claim, not
identity. Scope authorization leaves implementation acceptance and delivery
boundaries separate.

## Verification

`specsync change check <id>` materializes the approved delta. Cases may link the
requirements it adds or modifies before then while the checkpoint records the
approved change (see testing.md). `specsync change check <id>
--commit` also records targeted structural verification; run it after committing the
source with `.chough/`. Stage new source files first. It does not run the project's
tests; product checks are separate observations. It refuses an edited delta and
incomplete tasks. Fix real failures; if evidence is stale, follow the exact recovery
action status names.

## Review, archive, merge

Push only after the applicable local gates and observe required CI at the actual tip.
After the human reviews the change package, diff, delta and evidence:

```sh
specsync change review <id> --reviewer "<agent-account>"
specsync change ship <id>
```

Use the project agent account for the native record and retain the actual human
acceptance source separately; the actor label does not supply that acceptance. The
reviewer may be the scope approver. Do not commit between review and ship or
finalize. Archive on the same pull request, push the archive commit, wait for its
checks, and merge only when every change on that pull request is archived. Review and
finalize several changes one at a time.

Merge, package publication, release, deployment and live health are separate
observations; see [release.md](release.md).

## Evidence records

`chough cases run` and `chough evidence add` record with an observation the Git
revision, a working content identity, the environment, artifacts and, for cases, the
definition digest. Freshness compares the content identity, not the revision: it
covers every tracked or untracked non-ignored file except `.chough/`, `target/` and
SpecSync lifecycle bookkeeping (a change's `state.json`, `approvals.json`,
`verification.json`, `verification-attempts.json`, `review.json`,
`review-attempts.json`, `finalization.json`, `accepted-state.json`,
`lesson-bundle.md` and the front-matter `state:` value of `change.md`). So committing
only Chough records, an empty commit, `change check --commit` and finalize keep an
observation current, while a change to source, specs, configuration or a change's
deltas and other artifacts makes it stale. A record without a content identity, from
an earlier Chough, keeps the revision-bound comparison and is stale after any commit.
The records do not authenticate a witness or verify a remote artifact. A changed
input, artifact, case definition or environment makes the observation stale.

## Moves that already happened

When a stage moved outside Chough (for example in a subagent), record it with
`chough stage backfill --at <original time> --commit <rev> --quote "<the words it rests
on>"` rather than advancing now with "recorded after the fact" in a note. Native state
is still read now and every advance refusal applies. The quote is evidence only: it
never records a human approval or review, so record human gates and native approvals
the usual way when they happen.
