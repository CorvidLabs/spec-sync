---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: design
---

# Design

`git_utils` gains the precondition as a value rather than a convention:

    pub enum MissingHistory { NoRepository, NoCommits }
    pub fn missing_history(root: &Path) -> Option<MissingHistory>

`reason()` returns the machine strings and `headline()` the terminal strings,
both byte-identical to what #558 already emitted, so `stale`'s output does not
move while it is refactored onto the shared helper.

Each reader then refuses in the shape its own file already uses for an
unanswerable question:

- `report` mirrors its own coverage-inconclusive branch: inconclusive JSON on
  stdout under `--format json`, one line on stderr otherwise, exit 1. It sits
  *after* the coverage computation so an inconclusive coverage input still
  reports itself, and `stale_modules` is `null`, never `0` — a dashboard must
  not be able to record "not stale" for a project whose staleness is unknowable.
- `check --stale` was an explicitly requested measurement that silently produced
  zero warnings. It now emits the file's own inconclusive shape and exits 1.
  `"stale"` is `null` rather than `[]`, because an empty list is exactly the
  "no drift found" reading this change exists to remove. Plain `check` is
  untouched.
- The lifecycle `no_stale` transition guard passed when git could not answer,
  allowing a promotion on an unasked question. It now blocks.
- `scoring` does not refuse — a score is still useful without git — but it must
  not award points for an unmeasured dimension. `GitFreshness` records whether
  the git half was `Measured`, `NotApplicable`, or `Withheld`, and `Withheld`
  means the points were not granted. Consumers are told so they do not withhold
  them a second time.

The honest limitation, stated: this is a helper each reader must call, not a
choke point the compiler enforces. A fifth reader can still forget it. The
alternative — making `git_last_commit_hash`'s absent case impossible to read as
zero — is a wider change touching every caller of a `usize` return, and is worth
doing when that function is next opened. The four known readers are covered and
the integration matrix will catch a fifth that is not.
