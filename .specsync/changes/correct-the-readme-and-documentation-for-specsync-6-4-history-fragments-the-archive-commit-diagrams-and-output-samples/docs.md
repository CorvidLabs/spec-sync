---
change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
artifact: docs
---

# Docs

The pages changed: README.md, AGENTS.md (outside its managed trust block), docs/HLD.md (text and Mermaid diagrams 6, 8 and 10), docs/ADOPTING.md, site/src/content/docs/ (workflow.md, cli.md, architecture.md, cross-project-refs.md, quickstart.md) and site/src/content/examples/rust-workspace.mdx. Each change is one of the 21 items; the README's version and pin wording is not touched.

## The 21 items

The audit of main `3f0985e8` with its 6.4.1 build, as approved. Each item says what was wrong and where; this change corrects each one at that place.

### History fragments (6.4.0: no Change Log row, no `version:` bump)

1. `docs/HLD.md:329` says the first `change check` materializes deltas "(with a version bump and a Change Log row per module)". It writes `specs/<module>/history/<change-id>.md` and touches neither.
2. `docs/HLD.md:439` has the same claim in **Mermaid diagram 6** (the `change check --commit` sequence): "apply deltas, bump version, add Change Log row (one transaction)".
3. `docs/HLD.md:878` has it again in the glossary row "Materialize".
4. `site/src/content/docs/workflow.md:496-502`, "Keeping specs in sync", gives steps 3 and 4 as "Bump the `version`" and "Add a Change Log entry". Under the change workflow, `change check` records a history fragment instead. Without it, the Change Log and `version` are the author's own.
5. `AGENTS.md:125` says "Use `specsync compact` to keep changelogs manageable", and `workflow.md` §7 (around lines 518-529) says "As specs accumulate changelog entries… prevent unbounded growth". Since 6.4.0 changes add fragments, not rows. `compact` only matters for tables an older SpecSync wrote or that someone keeps by hand.
6. `README.md:265` says `*.spec.md` holds the "change history". Its Change Log is frozen, and the history is the `history/` fragments (`specsync history <module>`).

### The archive commit (`finalize` and `ship` commit it themselves)

7. `AGENTS.md:71`, the happy path's step 6, reads "commit the archive tip, push, …". `change finalize` and `change ship` already commit `chore(lifecycle): archive <id>` (`src/commands/change.rs:575-581` and `2795-2800`). The step is to push that commit. README:192 and HLD's diagram 7 already say so.
8. `docs/ADOPTING.md:111` reads "Commit and push the archive result", with the same fix.

### The journal's location (6.4.1: the user state folder)

9. `docs/HLD.md:597` (the `.specsync/` table) and `:828` (the failure-handling table) put the transaction journal in the "local state folder", which is HLD's name for the user cache location. 6.4.1 keeps it in the user state folder (`$XDG_STATE_HOME`, else `~/Library/Application Support` on macOS or `~/.local/state`), as HLD's own §5.5 and `cli.md:249` say.

### Mermaid diagrams against the workflows

10. **HLD diagram 8** (a pull request through CI and Trust, line 511) omits four `ci.yml` jobs:
    - `validate-action`;
    - `closed-late-compat` (runs SpecSync 6.0.0 and 6.2.0);
    - `linux-baseline`;
    - `attest` ("Record attestation"), which runs after the Required CI gate.
11. **HLD diagram 10** (the release, line 733) omits `release.yml`'s `authorize-release`. It runs between `validate` and `promote` and confirms the qualification run before anything is promoted.

### Commands and flags

12. `AGENTS.md:83` and `docs/ADOPTING.md:120`, `123` and `208` write `check --commit`, which reads as top-level `specsync check --commit`, which does not exist. It should be `change check --commit`.
13. `site/src/content/docs/cli.md` has no entry for `specsync merge-file`, the Git merge driver for contracts (`specsync merge-file %O %A %B %P`), or for `specsync change pr`, which only `workflow.md:15` mentions.

### Facts about the code

14. `AGENTS.md:171` lists "Requirements" among a spec's required sections. The default required sections (`src/types.rs:912`) are Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies and Change Log.
15. `site/src/content/docs/architecture.md:28` names `cli_args.rs`, which does not exist. Its dependency table also omits `fs2` (the lifecycle lock) and `cap-std` (the confined reads).
16. `README.md:14` and `:328` say "33 languages" but list 35 names. The code has 33 language families (`src/types.rs:687`): Lisp covers Common Lisp, Scheme and Emacs Lisp, and TypeScript covers JavaScript. Line 333 already says "families".

### Output samples (run, then compared)

17. `site/src/content/docs/cross-project-refs.md:80-85` shows `✓ CorvidLabs/spec-sync@cli — resolved`. The real `resolve --remote` prints a heading, "Remote Cross-project references:", then `✓ <spec> -> <repo>@<module>`, and `✗ … (module not in registry)` for a failure (`src/commands/resolve.rs:316-370`).
18. `site/src/content/examples/rust-workspace.mdx:26-31` shows output SpecSync never prints ("Reading 14 *.spec.md files across 3 crates…"), with an undocumented export as a `✗` error. In real output it is a `⚠` warning, in the per-spec format README's block shows.
19. `site/src/content/docs/quickstart.md:57-70` shows part of the `config.toml` that `init` writes as if it were the whole file. `init` also writes `exclude_dirs`, `exclude_patterns` and `enforcement`.

### Missing, where readers look for it

20. **`ship-status`'s exit code.** It exits 1 while it names a blocker. `cli.md:774` and `ADOPTING.md:165` say so; AGENTS.md's quick reference and `workflow.md`'s ship-status mentions do not.
21. **`change approve --dry-run`.** `cli.md` and AGENTS.md describe it; `workflow.md`'s approval step does not.
