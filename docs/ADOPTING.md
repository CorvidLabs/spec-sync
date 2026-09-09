# Adopting spec-sync

This page is written to be **pasted wholesale into an agent session** in the repository you
want to adopt spec-sync in. It is also readable on its own.

This guide describes the 6.0 workflow. Select an explicit release or candidate and verify the
installed binary before adoption; historical recovery guidance is labeled separately.

---

Adopt spec-sync in this repository.

spec-sync keeps markdown module specs in `specs/<module>/` synchronised with source code, and
gates changes through a lifecycle that records what was intended, what was verified, and who
reviewed it. Install it, generate specs for what already exists, wire CI, and drive one real
change end to end so the loop is proven rather than assumed.

## Install

Pin explicitly. After the stable release is published, install its source tag:

    cargo install --git https://github.com/CorvidLabs/spec-sync --tag v6.0.0 --locked specsync
    specsync --version

To try a pre-release instead, replace `v6.0.0` with the explicitly selected `v6.0.0-rc.N`
candidate tag. A source tag alone does not guarantee downloadable binary assets exist. Prebuilt
binaries exist for Linux and macOS only; 6.0 publishes no Windows binary (#735).

Coordinate the upgrade of all lifecycle writers to the selected 6.x version, including developer machines, agents, hooks, and CI. Older 5.x writers may reject slug-based changes or discard newer record fields. The 6.x downgrade checks detect damaged or downgraded evidence; they do not make mixed-version writes safe. Check `specsync --version` in each execution environment before resuming active work.

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

A successful check validates configured structure, exported API names, source mappings, dependency declarations, and supported schema rules. It does not prove that arbitrary natural-language requirements describe the implementation; reviewers and product tests establish those behaviors.

Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate.

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
    specsync change check <id> --commit
    # push the product tip after local gates; wait for required CI and complete human PR review
    specsync change review <id> --reviewer "<human>"
    specsync change ship <id>

Run `review` and `ship` consecutively, without committing between them. The reviewer may be
the scope approver. Commit and push the archive result, wait for required checks, then merge
on GitHub. **Merge only after every active change on the PR is archived.**

## 5. Wire CI

    # For a candidate instead, choose a release that has binary assets.
    - uses: CorvidLabs/spec-sync@v6.0.0
      with:
        version: '6.0.0'
        strict: 'true'
        lifecycle-enforce: 'true'

Both pins are needed and they pin different things: the `uses` ref pins the action code, the
`version` input pins the binary it downloads; prefer the exact tag over a floating major ref. For a
prerelease, replace both pins with the selected published candidate. The action runs on Linux
and macOS runners and refuses a Windows runner. When using a Trust wrapper, verify that its pinned revision
supports the requested SpecSync version and binary source. This repository passes its validated
candidate through an explicit version and runner-local mirror; an old wrapper may otherwise
continue using 5.x. A soft provenance setting alone does not enforce release provenance.

The action runs `specsync check` (and `specsync lifecycle enforce --all` when
`lifecycle-enforce` is set). It does not run the change-workflow audit. If you turned the workflow
on in §3 and want CI to gate on it, add a step that runs `specsync change audit`, and use
`fetch-depth: 0` on that job's checkout: the audit compares each change's verification commit
against HEAD's ancestry, and a shallow clone reports every change as orphaned.

`strict: 'true'` is a decision, not a default. Without it an undocumented export is a warning and
CI passes over drift. With it, drift gates.

## Things that will bite you, in the order they will

**Archive on the PR before merging.** Verification and scoped-review evidence have different
currency rules. A squash can make a historical review's ancestry unavailable even when verified
content still matches. That is a recovery concern for changes merged while still active, not a
reason to schedule fresh review after every normal merge. Complete review and finalization on
the PR first. Use `change ship-status <id>` for the current gate and `change status <id>` for
the supported recovery action if an older change was already merged prematurely.

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

    specsync change new "<summary>" --kind bug-fix \
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

**Recovery depends on the recorded workflow.** `reopen` also supports eligible accepted or
archived workflow-v2 changes with stale delivery evidence, including interrupted finalization
after terminal approval. Keep the approved definition unchanged, record the actor and reason,
then use `reopen` → `check --commit` → human review → `review` → `finalize`. Historical
`verify` / `accept` and classification/owner corrections have separate recovery guidance.
Use `change status <id>` and the [workflow recovery guidance](../site/src/content/docs/workflow.md)
for the state you actually have. Recovery does not replace finalizing before a normal merge.

**`db_tables` needs `.sql` migrations to be checkable.** If your schema lives in application code,
declare `db_tables` anyway — in 6.0 it is a notice, not a `strict`-gating warning. Point
`schema_dir` at your migrations only if you actually have them.

**A repeated description is refused by name.** Change identities are slugs derived from the
description, so two changes described the same way collide. The error names the existing one.

## Close the learning loop

The point of archival is not filing — it is that a module accumulates what was learned about it.

- At `change new`, spec-sync prints what the modules you declared already learned. **Read it before
  scoping**; it is there because a previous change paid for it.
- While building, put what you learn in the change's own `context.md` — prior attempts, dead ends,
  anything already ruled out.
- At `finalize` or `ship`, spec-sync writes a `lesson-bundle.md` into the archive.
  Folding useful lessons into a module's `context.md` is optional and is not a merge gate.
  If you choose to update tracked companions later, scope that documentation change normally;
  no recursive fold-back change is required merely because another archive contains lessons.

## Verify the adoption

    specsync check --strict          # 0 warnings, exports documented
    specsync change audit            # clean (the global --strict flag has no effect here)
    ls .specsync/archive/changes/    # your first change, archived

If all three hold, the loop is real. Report what you set up, what you had to correct in the
generated config, and anything the tool refused that you think it should not have.
