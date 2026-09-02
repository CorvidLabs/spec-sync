# Adopting spec-sync

This page is written to be **pasted wholesale into an agent session** in the repository you
want to adopt spec-sync in. It is also readable on its own.

Everything below was checked against `specsync 6.0.0` (`v6.0.0-rc.2`) — every verb, flag, and
path is one the binary actually accepts. The "will bite you" section is not speculative; each
entry is something hit while adopting spec-sync in a real repository.

---

Adopt spec-sync in this repository.

spec-sync keeps markdown module specs in `specs/<module>/` synchronised with source code, and
gates changes through a lifecycle that records what was intended, what was verified, and who
reviewed it. Install it, generate specs for what already exists, wire CI, and drive one real
change end to end so the loop is proven rather than assumed.

## Install

Pin explicitly. 6.0.0-rc.2 is a pre-release and will NOT be resolved by `latest`.

    cargo install --git https://github.com/CorvidLabs/spec-sync --tag v6.0.0-rc.2 --locked specsync

## 1. Initialise and generate

    specsync init
    specsync generate
    specsync check

`init` detects source directories and writes `.specsync/config.toml` and `.specsync/sdd.json`
with SDD **off**. `specsync check` is the product. Enable the change workflow later with
`specsync change adopt` if you want it. `generate` scaffolds a spec per module from the source it finds.

Read the generated `.specsync/config.toml` before going further. `init` guesses exclusions from
the language it detects, and the guess is often wrong for a repo that mixes languages — a Swift
package can come out with TypeScript test exclusions. Fix `exclude_patterns` and `source_dirs`
now, not after the first hundred warnings.

## 2. Fill the specs, then set them active

A generated spec is `status: draft`, and a draft spec SKIPS section and export validation.
`check` says so:

    ⚠ Spec is `status: draft` — section and export validation were skipped

That is the honest default for spec-first authoring, and it means a draft spec proves nothing.
Once a module's spec describes real behaviour, set `status: active` and re-run `check`. You
should see `N/N exports documented`. If you do not, the spec and the code disagree, which is the
thing this tool exists to tell you.

Fill the companion files too — `context.md`, `requirements.md`, `testing.md`, `tasks.md`. They
are where a module accumulates what was learned about it. Unfilled scaffold markers are warnings
until you run `--strict`, at which point they gate.

## 3. Turn the change workflow on (optional)

`init` left `.specsync/sdd.json` with `enabled: false`. Skip this section if `specsync check`
in CI is all you want. To drive changes through the lifecycle in §4:

    specsync change adopt

That flips `enabled` and nothing else. `require_change_for_meaningful_files` stays `false`, so
editing a source file without an active change is not an error; set it to `true` by hand if you
want `change audit` to demand one.

`change check` does not run your tests. It compares the specs this change owns or maps to the
code, in-process, and records that as evidence. `verification_commands` is still accepted in
`sdd.json` for adopters who list it, but nothing executes it — `cargo test` / `swift test` /
`bun test` belong in CI, next to `specsync check`.

## 4. Drive one real change end to end

Do not skip this. It is the only way to find out whether the setup is right.

    specsync change new "<a real thing you are about to do>" --kind feature --spec <module> --path <file>

`change new` prints an interview. Answer every question, then fill the artifacts it selected in
`.specsync/changes/<id>/` — the selection is adaptive, so a low-risk change gets fewer.

    specsync change answer <id> acceptance_criteria "<what observable outcome proves this is done>"
    specsync change answer <id> public_contract yes|no
    specsync change answer <id> architecture_risk yes|no

Write a `deltas/<module>.md` describing the requirement being added or changed, then:

    specsync change approve <id> --actor "<you>"
    # …write the code…
    specsync change check <id>
    specsync change check <id> --commit
    specsync change review <id> --reviewer "<human>"
    specsync change ship <id>

Then commit and merge. **Merge only after `ship`** — merging first orphans the verification
evidence, and the tool will tell you so.

## 5. Wire CI

    - uses: CorvidLabs/spec-sync@v6.0.0-rc.2
      with:
        version: 6.0.0-rc.2
        strict: 'true'
        lifecycle-enforce: 'true'

Both pins are needed and they pin different things: the `uses` ref pins the action code, the
`version` input pins the binary it downloads.

The action runs `specsync check` (and `specsync lifecycle enforce --all` when
`lifecycle-enforce` is set). It does not run the change-workflow audit. If you turned the workflow
on in §3 and want CI to gate on it, add a step that runs `specsync change audit`, and use
`fetch-depth: 0` on that job's checkout: the audit compares each change's verification commit
against HEAD's ancestry, and a shallow clone reports every change as orphaned.

`strict: 'true'` is a decision, not a default. Without it an undocumented export is a warning and
CI passes over drift. With it, drift gates.

## Things that will bite you, in the order they will

**Your merge strategy decides what a landed change still owes you.** Verification evidence and
the scoped review are both recorded against a commit hash. **A squash-merge rewrites that
hash**, so anything that reads history across the merge loses its footing.

What a squash costs you is the **scoped review**, and only that. The review check walks the
commits between the review and `HEAD` to prove nothing changed except the change's own records,
and a squash makes that walk impossible rather than merely false — so expect to record a fresh
review after a squash. The reviewer MAY be the same human who approved the definition.
CorvidLabs/spec-sync#694 tracks it, and it needs a decision about what a review proves rather
than a patch.

What a squash **no longer** costs you is re-verification. Ship readiness stopped depending on the
recorded commit being reachable; it asks whether the recorded plan and tree still match what was
verified, which is true under every merge strategy. Measured across all three: squash, rebase and
merge-commit each reach `ready to finalize`.

Check your repository's settings before you adopt, not after:

    gh api repos/OWNER/REPO --jq '{merge:.allow_merge_commit, squash:.allow_squash_merge, rebase:.allow_rebase_merge}'

If rebase-merge is disabled and squash is the only option, **you will hit this every time**, and
`gh pr merge --rebase` will silently fall back to squash without telling you.

**There is no configuration that avoids it, and no advice worth giving.** spec-sync's own
repository is squash-only (`merge: false, rebase: false, squash: true`), and only **21 of its 198
archived changes — 11% — still have a reachable verification commit**. Telling you to rebase-merge
would be telling you to do something the tool's own repository cannot do.

**Merging before `finalize` costs more than it says.** It orphans that change's evidence — and it
also blocks **every earlier accepted change sharing a delivery input** from archiving, until the
merged one is finalized or those predecessors are reopened. One early merge can stall an unbounded
set of older changes. Finalize first, every time.

**Scope freezes at approval, not at creation.** You can widen `affected_specs` and
`affected_paths` while the change is `draft`. Once approved you cannot, and there is no withdraw
verb — a mis-scoped change past approval has no clean exit. Get the scope right before
`approve`, and if you are unsure, look at what the change actually touches first.

**Every path you touch needs an owning module.** Production source declared under
`--no-spec-change` is refused: that flag means "no spec text changes", not "no module owns this".
The refusal arrives at `ship`, several stages after the only place you could have fixed it —
and scope freezes at approval, so by then the only exit is to redo the change.

The remedy is to declare the owning specs **and** `--no-spec-change` together. They are not
mutually exclusive, which is not obvious:

    specsync change new "<summary>" --kind fix \
      --spec change --spec cmd_change \
      --path src/change.rs --path src/commands/change.rs \
      --no-spec-change --rationale "behaviour only, no spec text changes"

Find the owning spec for a path by grepping the `files:` list in each `specs/<module>/*.spec.md`.
A path with no owner is itself the problem — fix that before creating the change.

**The reviewer may be the approver.** Scoped review records who signed off. It does not invent a
second person. If a repository wants two humans, that is GitHub required reviewers, not SpecSync.
`Alice` may review what `alice` approved.

**Build directories will wreck your verification.** Add `.build/`, `target/`, `node_modules/` to
`.gitignore` before the first `change check`. Their churn moves the workspace digest and stales
evidence you just recorded.

**Making a symbol more visible is a contract change.** Widening something to `pub(crate)` makes
it an export the spec must document. This is the drift check working, not a bug.

**Two state traps, with escapes that are not in the error text.**

- `check --commit` then `accept` can deadlock on a reopened workflow-v1 change: the
  verification-recording commit leaves `HEAD` one ahead of the evidence it records, and looping
  `check --commit` never converges because each run recreates the condition. **Escape: run `check`
  WITHOUT `--commit`.**
- `ship` refuses a stale review while `review` refuses to run in `accepted` state, so the verb that
  would fix it is unreachable from the state you discover it in. **Escape: `finalize` rather than
  `ship`.**

**`db_tables` needs `.sql` migrations to be checkable.** If your schema lives in application code,
declare `db_tables` anyway — as of 6.0.0-rc.5 it is a notice, not a `strict`-gating warning. Point
`schema_dir` at your migrations only if you actually have them.

**A repeated description is refused by name.** Change identities are slugs derived from the
description, so two changes described the same way collide. The error names the existing one.

## Close the learning loop

The point of archival is not filing — it is that a module accumulates what was learned about it.

- At `change new`, spec-sync prints what the modules you declared already learned. **Read it before
  scoping**; it is there because a previous change paid for it.
- While building, put what you learn in the change's own `context.md` — prior attempts, dead ends,
  anything already ruled out.
- At `finalize` — and at `ship`, which finalizes for you — spec-sync writes a `lesson-bundle.md`
  into the archive, and both verbs name the step ahead of their remaining guidance:

      Next: write lessons into specs/<module>/context.md from <archive>/lesson-bundle.md,
            then merge the PR on GitHub

  **Do that before merging.** A change's own `context.md` is archived and read by nobody; the
  spec's `context.md` is read before every future change to that module. This is the only point in
  the lifecycle where knowledge compounds instead of merely being recorded.

**The fold-back is itself a change, and the obvious scoping makes it recurse.** Writing into
`specs/<module>/context.md` touches tracked paths, so the fold needs its own lifecycle record.
Declare that record against the modules it edits — the obvious thing to do — and its own
`finalize` hands you the same instruction for the same specs, which needs another change, which
gets the instruction again. There is no cycle detection and no warning; the instruction just never
stops.

What stops it is declaring no specs at all. A change with no affected specs owns no modules, so
there is nothing to fold into, and both verbs print their plain merge guidance instead:

    specsync change new "Fold the <topic> lessons into the <module> contexts" \
      --kind documentation --path specs/<module>/context.md \
      --no-spec-change \
      --rationale "fold-back of an archived lesson bundle; no spec text or behaviour changes"

Note what is absent: no `--spec`. That is the mechanism. `--no-spec-change` on its own is not
enough, because it coexists with `--spec` (above), and a fold that declares specs is told to fold
again. The rationale has to be true of this change in particular — it folds an already-archived
bundle into `context.md` companions and alters no canonical spec text, requirement, or behaviour.
If that is not true, it is not a fold-back and must not be scoped as one.

Keep such a change to `context.md` paths. A spec companion is not production source, so it does
not trip the owning-module refusal above; production source in the same change does — and it would
have lessons of its own to fold. Fold separately.

Measured on this repository: 6 of 183 archived changes have ever touched a spec's `context.md`.
The fold-back is the step that gets skipped.

If a module's lessons grow long, treat that as a signal the module is too large or too hot — not as
something to compact away.

## Verify the adoption

    specsync check --strict          # 0 warnings, exports documented
    specsync change audit --strict   # clean
    ls .specsync/archive/changes/    # your first change, archived

If all three hold, the loop is real. Report what you set up, what you had to correct in the
generated config, and anything the tool refused that you think it should not have.
