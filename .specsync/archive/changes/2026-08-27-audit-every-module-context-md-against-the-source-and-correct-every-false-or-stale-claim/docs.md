---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: docs
---

# Docs

The 34 edited `specs/<module>/context.md` files ARE the documentation deliverable. No user-facing
documentation changes: no README, no `docs/`, no CLI help, no canonical spec text.

## Files edited

`agents`, `change`, `changelog`, `cmd_agents`, `cmd_check`, `cmd_comment`, `cmd_coverage`,
`cmd_deps`, `cmd_diff`, `cmd_hooks`, `cmd_init`, `cmd_init_registry`, `cmd_new`, `cmd_report`,
`cmd_scaffold`, `cmd_score`, `cmd_wizard`, `commands`, `comment`, `deps`, `exports`, `git_utils`,
`github`, `hooks`, `ignore`, `importer`, `manifest`, `mcp`, `merge`, `output`, `parser`,
`registry`, `schema`, `validator`.

Twenty-eight `context.md` files were audited and needed no correction: `ai`, `archive`, `cli`,
`cli_args`, `cmd_archive_tasks`, `cmd_change`, `cmd_changelog`, `cmd_compact`, `cmd_generate`,
`cmd_import`, `cmd_issues`, `cmd_lifecycle`, `cmd_merge`, `cmd_migrate`, `cmd_resolve`,
`cmd_rules`, `cmd_stale`, `cmd_view`, `compact`, `config`, `generator`, `hash_cache`, `rehash`,
`scoring`, `types`, `util`, `view`, `watch`.

## Found while auditing, NOT fixed here — each wants its own issue

**1. `AGENTS.md:51` contradicts itself in one bullet.** It reads
"`.specsync/change-sequence.json` is **read-only history**. Nothing writes it." and then, in the
same sentence, "`change check` raises a stale working-tree copy to the committed mark and says so."
Both halves cannot be true. The second is closer: `floor_sequence_ledger_to_committed` writes the
file — but from `git_commit_all`, so it runs on lifecycle commits, not on `change check`. This
wording arrived with #732, which was itself fixing stale allocation text. Out of scope: `AGENTS.md`
is not a `context.md` and is not in this change's declared paths.

**2. `src/change.rs:1802-1806` still documents the deleted allocation model.** The doc comment on
`floor_sequence_ledger_to_committed` says "`change new` writes it into the working tree only" and
"the next allocation can hand out an ID that is already taken". Nothing allocates since #665, so
both sentences describe a mechanism that no longer exists. This is the same defect shape #732 swept
out of the spec text, surviving in the source comment the sweep did not reach — and it is the
comment sitting directly above the function whose existence falsifies the `context.md` claim this
change corrects. `maximum_observed_sequence`, which that comment cites, exists nowhere else in the
tree. The same sentence is also user-facing: `src/change.rs:2189` prints "nothing writes this file
any more" in a repair diagnostic. Its actual point — that you cannot repair the ledger by
allocating — is true; the clause carrying it is not.

**3. `docs/ADOPTING.md:234` carries a decaying measured count**: "6 of 183 archived changes have
ever touched a spec's `context.md`". There are 202 archived changes now. Same class as the counts
corrected here, in a file outside this change's scope.

**4. `specs/cmd_score/cmd_score.spec.md`'s Public API table omits `min_score: Option<u32>`**, which
the live `cmd_score` signature takes. `specs/cmd_score/context.md` now says so; correcting the
canonical spec is a spec-text change and needs its own scoped record.

**5. `tests/integration/comment.rs::comment_suppresses_configured_command_output_but_check_streams_it`
is now vacuous.** It asserts `comment` does not print configured-command output. Since #543
severed `comment` from the trust layer, `comment` runs no configured command at all, so the
assertion passes for a reason unrelated to its name. Not repaired here — it is a test change, not a
documentation one — but `specs/cmd_comment/context.md` now records what it actually proves.
