# Checkpoint and cold resume

Essential state must never live only in model context. `.chough/session.json` is the
single writable checkpoint. It works without the librarian and without any other
storage adapter.

## What it holds

| Field | Content |
|---|---|
| `objective`, `stage` | Current outcome, stage, status, side-stage callers and iteration |
| `native_change` | The SpecSync change ID; native status decides its state |
| `references` | Criteria, requirements, cases, design sources and files in scope |
| `decisions`, `open_questions` | Decisions with their reason in the sentence; questions that matter |
| `completed`, `observations`, `bugs`, `blockers` | Work done and the records that show it |
| `awaiting_human` | Real gates still waiting for a person |
| `run_consents` | Consent for this objective only |
| `next_action` | Suggested next step; never overrides native state |

Standing preferences are in `.chough/preferences.json`. Legacy transitions stay in
`.chough/transitions.jsonl`; adopted and new projects use immutable events under
`.chough/history/cycles/`, selected by `history_cursor` in the same checkpoint.
Observations are in `.chough/evidence/`, bugs in
`.chough/bugs/`, case definitions in `.chough/cases/` and retention removals in
`.chough/pruned.jsonl`.

## When to save

Save after each meaningful increment, before handing off, before context is cleared,
and whenever a human gate opens or closes:

```sh
chough checkpoint --completed "<what was done>" --observation <id> \
  --decision "<decision, because reason>" --next-action "<next step>"
```

Opening a gate with `--awaiting-human "<gate>"` appends one `gate-opened` record to
`.chough/transitions.jsonl` (only when that gate is not already open), and
`--clear-awaiting-human` appends one `gate-closed` record per gate it clears. They
record when a gate waited, never who decided or what, and `chough timing` reads them
to show waiting-on-human time. Open and clear gates through these flags rather than by
editing `session.json`, or the waiting time is lost.

`--clear-awaiting-human` clears every open gate, not just one. To close some gates
and keep the others, pass the gates still waiting with `--awaiting-human` in the same
call; they are logged closed and opened again, so their waiting time restarts there.

Write decisions with a prefix and a reason, so a fresh agent can tell what was chosen,
what was ruled out and what already failed:

```text
decided: <what> because <reason>
rejected: <approach> because <reason>
failed: <experiment> because <reason>
```

This is a convention, read best-effort: the prefix starts the entry, lowercase, with a
colon, and the reason is the text after the last ` because `. Any other entry, such as
older free text or a dated note, is still kept and shown whole. Decisions stay plain
strings in the checkpoint; nothing validates the convention. Record a human's decision
only in their words, and label an agent's own assumption as the agent's, without a
`decided:` prefix.

Write decision summaries and observations, not private reasoning transcripts.
SpecSync's `Handoff:` line says whether its own records are safe to clear; follow it
too.

## Resume cold

A fresh agent with none of the earlier conversation:

1. Reads `AGENTS.md`, then this skill.
2. Runs `chough next --environment <env>` (or reads the files above directly); it is
   the focused view, and `chough status` is the full one. Besides the stage, native
   state, next action and gates, it shows the 5 newest decisions with their kind and
   reason, the latest failed observation by recorded time with the status of each bug
   that links it, and where each module's lessons live (`<module>.3md`, or the
   `## Lessons` line of `context.md`) with the newest archived SpecSync lesson
   bundles. It points to lessons and never prints them; read the source it names.
3. Checks native status for `native_change`, freshness of the referenced observations,
   open blocking bugs and `awaiting_human`. A referenced observation is `current`,
   `historical` (present but stale), `pruned` (removed by retention, in the ledger) or
   `unavailable` (missing, not in the ledger); only a current one counts, so rerun the
   others rather than treat them as passed.
4. Continues with `next_valid_action` under the same scope, preferences and gates. It
   never records a pending human decision itself. In stage done with the native
   change archived, the cycle is closed: `chough next` recommends `chough stage reset
   --to intent --reason "<why>"` (or `--to plan` when the approved intent already
   covers the new work), then `specsync change new "<outcome>" --spec <module> --path
   <path>` and `chough checkpoint --native-change <id> --next-action "<next step>"`.
   The same holds at an earlier stage when the native change is already archived:
   `chough next` names the archive and the new cycle instead of continuing, after
   `chough stage advance` to done at accept. A reset replaces the checkpoint's next
   action.

## Schema and concurrency

A schema 1 session keeps working for `context` and `status`. `chough migrate`
converts it after saving the original bytes to `.chough/session.v1.json`. Hand edits
are allowed and validated on every read. With several agents, one agent owns live
checkpoint writes; others report to it. APS was evaluated as a backend and deferred:
it lost concurrent read-modify-write updates without error and accepts unknown fields.

For committed parallel schema 2 edits, the repository declares:

```gitattributes
.chough/session.json merge=chough-session
```

Each clone that wants automatic merging configures the driver locally:

```sh
git config --local merge.chough-session.name "Chough schema 2 checkpoint"
git config --local merge.chough-session.driver "chough checkpoint --merge %O %A %B"
```

Git supplies base, ours and theirs as `%O`, `%A` and `%B`. `chough init` never sets
these values. Without a driver, use the explicit fallback:

```sh
chough checkpoint --merge <base> <ours> <theirs>
```

The merge preserves ordered exact-string history and reference changes. It treats
objective, intent scope, modules, stage, native change, next action, run consents,
blockers and awaiting-human gates as one current-cycle group. If both sides change
that group differently, leave the conflict unresolved and ask the checkpoint owner
or human to select a coherent cycle. Never hand-resolve it by combining fields. A
merge is not approval, consent, gate closure or a lifecycle transition.

Success replaces only ours. Every refusal preserves all three inputs and writes no
Chough record. The merge adds no second checkpoint store and does not change the
one-owner rule for live writes.

## Immutable transition history

New projects use event history. Existing schema-2 projects remain in legacy mode
until the owner previews `chough history adopt` and explicitly runs
`chough history adopt --apply`. Adoption preserves the legacy log and schema-1
migration backup byte-for-byte. A digest-bound anchor detects later legacy changes.
An older helper may reject an adopted checkpoint; use a compatible helper and never
remove its cursor to make an old reader accept it.

The optional `history_cursor` is part of the atomic current-cycle merge group.
Immutable event paths from independent branches can integrate, while divergent
cursors require selecting one coherent checkpoint. Other branch events and orphan
writes remain visible in `chough history show`; they do not prove a completed stage.
Only parent links on the selected lineage order current work. Native gates still
decide lifecycle transitions.

Recent guidance follows at most 128 events and 512 KiB. Local appends to a new cycle allow at most 4,096
events and 16 MiB; a capacity refusal preserves the checkpoint, and a reasoned
`chough stage reset` can begin a new cycle without deleting the old proof. Timing
bounds transition content at 4,096 events / 16 MiB and keeps other tips separate
with unknown durations. Missing or intentionally pruned history is partial, never
zero or verified.

`chough history prune` is a read-only preview. It preserves active or referenced
cycles, unresolved branches/orphans, the newest three closed cycles including time
ties, and cycles younger than 14 days. Protected proof can prevent reclamation.
`--apply` requires both an explicit history-prune command and human retention
authorization; evidence auto-retention never starts history pruning. Maintenance
has one 16,384-read / 64-MiB metadata budget, including revalidation. An incomplete
scan refuses apply, and an interruption leaves an immutable receipt with an
explicit partial result. Applying adoption or cleanup to a real project is a
separate human operation, never a side effect of finishing implementation.

Before integrating a branch that pruned history, check the other branch's new
references and retained hashes. If new proof references removed content, restore
the exact bytes from Git before accepting integration; a pruned or unavailable
reference is not passing evidence. The one-owner live checkpoint and maintenance
rule still applies; arbitrary external filesystem writers are not serialized.

Git may combine independently written event sets above that local capacity. Retain
all imported proof, refuse further ordinary appends to the full cycle, and use a
reasoned reset for new work. The limit never silently discards merged events.
