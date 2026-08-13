---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: context
---

# Context

## What led here

Preparing `v6.0.0-rc.1` for team adoption, the release promise was scoped to the
*product surface* — `check`, `coverage`, drift detection, export extraction — while
the lifecycle verbs finish their reduction. That promise only holds if a cold agent
or a new engineer can pick up the binary and get a green repository. Three defects
sit directly on that path, and all three were found by walking it rather than by
reading code:

1. **`specsync init` leaves a repository that fails `specsync check`.** Every file
   `init` writes (`.specsync/config.toml`, `.specsync/version`, `.specsync/sdd.json`)
   is a protected SDD path. `path_is_meaningful_with_specs` returns `true` from
   `is_protected_sdd_path` *before* the ignore filter runs, so the first commit after
   initialization lands as uncovered meaningful delivery. The gate is unsatisfiable by
   construction: it demands a change workspace that covers files written before any
   workspace could exist.

2. **`specsync scaffold` writes prose that `specsync check` rejects.** Scaffolding a
   module emits placeholder sections; the effective-contract gate treats an unfinished
   section as a stub warning, and under `--strict` that is fatal. The tool's own output
   fails the tool's own gate.

3. **A directory in a spec's `files:` block makes `check` silently green (#472).**
   Filed as Kotlin-specific; it is not. A directory mapping extracts zero exports, so
   the Public API comparison has nothing to compare and passes. `check --strict --force`
   exits 0 with zero warnings while measuring nothing — the worst failure mode a
   verification tool has, because the signal it emits is indistinguishable from success.

## What a session picking this up needs to know

- **Quill is Rust.** These three are the RC blockers for a Rust consumer. #479 (Ruby
  visibility leakage), #529, #474, #477, and #473 do not block it and ride to `rc.2`.
- **Delivery scope freezes at the interview and cannot be widened** (#542). The full
  suite was run against the finished implementation *before* `change new`, so the
  declared scope is the measured blast radius, not an estimate.
- **These three fixes were batched into one change deliberately.** Running them as
  three parallel changes cost roughly a million tokens and eleven agent-hours, almost
  all of it in `change check --commit` retry loops contending on the same target
  directory. Verification is ~18 minutes uncontended and considerably worse under
  parallelism.
- **Fix 1 and fix 2 interact.** The bootstrap exemption adds a path to
  `uncovered_meaningful_paths`, while the scaffold fix changes how
  `validate_effective_contracts` splits warnings from errors. Both feed `check_project`.
  A fresh `init` → `scaffold` → `check` sequence exercises both at once, which is
  exactly the sequence the RC promises works.

## Ruled out

- **Adding the protected SDD paths to the default ignore list.** It would fix the
  symptom and disarm the guard permanently: later hand-edits to `sdd.json` would stop
  being meaningful delivery, which is precisely what the guard exists to catch.
- **Exempting all stub warnings for scaffolded modules.** A section a change actually
  authored and then emptied must stay fatal. The exemption is keyed to authorship, not
  to content shape.
- **Expanding a directory mapping into its files during validation.** Rejected as a
  silent behavior change: a spec that declares `src/provider` would begin asserting a
  Public API it never wrote. The directory is reported as an error with the expansion
  offered as the *fix*, so the author opts in.
