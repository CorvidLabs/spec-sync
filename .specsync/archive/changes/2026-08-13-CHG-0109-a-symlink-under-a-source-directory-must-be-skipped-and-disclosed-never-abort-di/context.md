---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#546, found during RC-era stress testing and verified by hand from a
minimal fixture. A single symlink under a source directory — pointing *inside* the
project, referenced by no spec — killed the entire command:

```
$ specsync check
SpecSync discovery is inconclusive: Coverage source-detection path src/alias.rs must not traverse a symlink or reparse point
exit 1
```

That one line was the whole output. No per-spec validation, no coverage numbers, no
summary. `coverage`, `score`, and `generate` behaved the same way. The
`src/generated -> ../build/gen` pattern, and monorepos that link shared source into a
package, made spec-sync unusable on those repositories.

## Why skipping is correct and not a weakening

The guard exists because these walks run behind retained directory capabilities and use
`symlink_metadata` so a link can never redirect discovery outside the retained root.
`manifest.rs:4748` pins that. **Resolving the target and re-checking containment would
reintroduce exactly that escape and is TOCTOU-prone — that is not the fix.**

Skipping loses nothing real, in both possible cases:

| link target | consequence of skipping |
|---|---|
| **outside** the project root | must not be followed anyway — this is what the guard is for |
| **inside** the project root | already discovered and counted under its real path; following would double-count |

The walk therefore never needs to traverse a link to be complete. Aborting bought no
correctness, only a denial of service on the whole command.

## The hazard this change had to avoid

Silently skipping shrinks the coverage denominator. A repository whose `src/vendor` is a
symlink would report a **higher** percentage after this change than before — a number that
improved because measurement stopped. That is the exact failure class the current hardening
work exists to eliminate, so disclosure is mandatory rather than a nicety.

This was not hypothetical: the first working implementation completed the run and printed
`File coverage: 1/1 (100%)` with no mention of the skipped link. It was caught by testing
against the acceptance criteria rather than against "does it still abort".

## What a session picking this up needs to know

- **Spec-tree symlinks stay fatal, deliberately.** Skipping a symlinked *source* file loses
  nothing because its real path is walked anyway. Skipping a symlinked *spec* silently drops
  a whole spec from validation — a much larger hole, and one nobody would notice.
- **A configured `source_dirs` entry that is itself a symlink also stays fatal.** Skipping
  something discovery merely encountered loses nothing; silently skipping a source tree the
  author explicitly asked to be measured is the failure this change exists to prevent.
- **Exit logic is duplicated** across `compute_exit_code` and `exit_with_status`. Patching
  only the first left `--strict` silently passing, and it took an isolated fixture to
  notice. Anything touching gating must change both.
- `cargo build --release` does not compile `#[cfg(test)]`. It went green while eleven test
  construction sites of `CoverageReport` were missing the new field. Use `cargo test
  --no-run` for discovery.
