---
change: add-a-hi-check-gate-to-ci
artifact: context
---

# Context

This repository now records its product intent with `hi` (human intent): 159 criteria
across 11 families under `hi/`, plus `INTENT.md`. Those files are the layer above the
canonical specs. A criterion is a plain sentence with a permanent id, and the one thing
`hi` guarantees is that an id is permanent and never reused.

Nothing was validating them. `hi check` catches exactly six structural problems: a
duplicate id, a case with no parent, an id colliding with a retired one, an id-shaped line
that is not a valid id, a family a file never declared, and a criterion stranded outside
every section. Three of those can otherwise go unnoticed indefinitely, because a broken
`hi/*.md` still renders as perfectly ordinary markdown in review.

What a session picking this up needs to know:

- `hi check` never fails on unfinished intent, only on a structurally broken file. A
  criterion that is false today is correct `hi`: it means the code has not arrived yet.
  This gate therefore cannot go red because somebody wrote down a want that is not built.
- The pin matters. `hi` 0.5.0 changed what `hi check` reads: a file in `hi/` whose name is
  not lowercase is hi's own rather than criteria. Pinning an older version would check
  these files with rules the tool no longer has, so the install is pinned to `0.5.0` with
  `--locked` rather than floating.
- `hi-check` also belongs in the corvid-pet job's `needs` and status table. Otherwise a
  structurally broken `hi/*.md` can still receive a passing scoped-review comment while
  `implementation-gate` is the only thing that blocks merge.
- `Cargo.lock` is a meaningful path. rustls 0.23.38 is RUSTSEC-2026-0285 (upgrade to
  >=0.23.45); without that bump every full CI run fails `cargo audit` before the new gate
  can be the story.
- `.github/` is a meaningful path under `require_change_for_meaningful_files`, which is why
  this change record exists. The lifecycle gate found the uncovered path before merge.
