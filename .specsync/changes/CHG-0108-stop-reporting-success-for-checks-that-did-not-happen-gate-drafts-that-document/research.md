---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: research
---

# Research

## How these were found

Two read-only agent fan-outs against a binary built from the CHG-0107 branch — adversarial
repository shapes, and seven real CorvidLabs repos — plus a hand-walked cold-agent path.
Agents were forbidden from running any `change` verb or `cargo`, so they exercised only the
product surface.

**Every finding was re-verified by hand from a minimal fixture before being acted on, and
two did not survive that.** Recording both, because a wrong finding acted on would have
made the tool worse:

- **"`--strict` deletes the drift-warning class instead of escalating it."** The numbers
  reproduced exactly — 33 warnings under bare `check`, 0 under `--strict`, on two
  independent fresh clones. The conclusion was backwards. `src/commands/check.rs:273`
  empties the classification list under `force || strict || !spec_filters.is_empty()`
  because those modes re-validate everything and do not need to know what changed; the
  warnings that vanish are the cold-cache artifacts of finding 3. `--strict` was producing
  the *more* correct output. Acting on the report as written would have pushed 33
  meaningless warnings into every CI run.
- **"The draft skip is silent."** It is not. Three separate notices name it:
  `⊘ Section validation skipped (status: draft)`, `⊘ Export validation skipped`, and a
  summary `ℹ N draft spec(s) skipped …`. The vacuous pass is real; the silence was not.

## Finding 1 — the draft gate

The decisive question was whether a draft with present source is *supposed* to pass
`--strict`. Reading the three tests that pin it settled it:
`draft_planned_mapping_passes_strict_and_is_absent_from_coverage` exists to prove
spec-first authoring works — write the spec, then the code — and its tail deliberately
creates the source file to check coverage accounting transitions to 2/2.

So the contract protects spec-first authoring, and its tail was about coverage, not about
blessing an unvalidated spec. The distinguishing evidence:

| | files present | Public API | in the pinned tests |
|---|---|---|---|
| spec-first | no | either | yes — must keep passing |
| honest stub | yes | empty | fixture is exactly this |
| opting out | yes | named symbols | not covered by any test |

The helper's Public API tables are headers with no rows. That is what made the narrow rule
possible: it lands entirely in the row no existing test occupies.

Confirmed against reality — `3md` (3 specs, ~63 public declarations) and `attest` (1 spec,
13 Swift files, 124 undocumented exports) are both draft-with-present-source-and-documented-API,
both report 100% coverage, and both currently pass.

## Finding 2 — quoting

Traced to `src/parser.rs:312`, where a block list item is pushed after comment stripping
and nothing else. `parse_flow_string_list` already unquoted its items, so flow-style and
block-style disagreed. Reading `set_scalar` showed scalars had the defect too —
`module: "auth"` and `status: "active"` both retain their quotes — which moved the fix from
the `files:` handler to the parse layer.

`strip_yaml_comment` returns quoted values untouched, so ordering matters: unquote *after*
comment stripping, and handle a comment that follows the closing quote inside the unquoter.

A second `files:` parser lives in `hash_cache::extract_frontmatter_files`, written to avoid
a circular dependency. Left alone it would have cached the quoted literal, so a spec with
quoted paths would look changed on every run.

## Finding 3 — cold-cache drift

`HashCache::is_changed` returns `true` for an absent entry, commented `// new file`. Correct
for selection, wrong for reporting. `.specsync/hashes.json` is untracked — confirmed with
`git ls-files --error-unmatch` — so CI is always cold and always noisy.

Verified the fix preserves the signal, not just the silence: on a warm cache, appending a
line to `specs/ai/requirements.md` produces exactly one correctly-named warning.

## Finding 4 — the remediation line

Introduced by CHG-0107. One `--path` per changed file on one line; a wide refactor produced
over 8000 characters.

## Not fixed here — #546

A symlink under a source directory, pointing inside the project, referenced by no spec,
aborts `check`/`coverage`/`score`/`generate` with a single line and no validation output.
Verified by hand.

The rejection sites (`src/validator.rs:424`, `:490`, `:714`, `:1045`, `:1251`, `:4928`,
`:4960`, plus `manifest.rs`) all sit behind retained directory capabilities and use
`symlink_metadata` precisely so a link cannot redirect discovery outside the root;
`manifest.rs:4748` pins that. Resolving targets would undo it. Deferred with a design note
rather than patched under time pressure.
