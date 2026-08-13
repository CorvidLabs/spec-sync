---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: design
---

# Design

Four mechanisms. Three of them make the tool stop claiming something it has no evidence
for; the fourth stops it discarding evidence it does have.

## 1. The draft gate is three-way, not two-way

`ValidationResult` gains two observations, both recorded even when section and export
validation are skipped:

| field | meaning |
|---|---|
| `had_present_source` | at least one mapped file resolved to a readable file |
| `documents_contract` | the Public API section names at least one symbol |

`had_present_source` is set only in the two branches that reach a real readable file — the
`SourceSnapshot::Present` arm and the ambient `full_path.is_file()` arm. Missing, planned,
directory, unreadable, and escape mappings all fall to earlier branches and never set it,
so REQ-validator-011 holds by construction rather than by a separate check.

`documents_contract` is `!get_spec_symbols(body).is_empty()`, computed before the section
gates so the draft path reaches it.

The warning fires only on the conjunction:

```
draft + files absent            -> pass   (spec-first authoring)
draft + files present + no API  -> pass   (honest stub, claims nothing)
draft + files present + API     -> WARN   (--strict exits 1)
```

**Why the conjunction rather than `had_present_source` alone.** Three integration tests
pin "a draft passes `--strict`", including with its source present. Their fixture's Public
API tables are headers with no rows, so the conjunction leaves all three passing
*unedited*. A rule that requires no pinned contract to be rewritten is a rule that did not
change the contract — it named a case the contract never considered.

Bare `check` stays exit 0 throughout: this is a warning, and warnings gate only under
`--strict`.

## 2. Drift is reported only against a baseline

`ChangeClassification` gains `baseline_known`, set from a new
`HashCache::has_baseline(rel)` for the spec's own path, and a `reportable(kind)` helper
that is `baseline_known && has(kind)`.

`is_changed` still treats an absent entry as changed — that is correct for **selection**,
because with no baseline everything must be re-validated. `check.rs` switches both
reporting sites (`Requirements` and `Companion`) from `has` to `reportable`, so the same
specs are validated and only the unsupportable claim disappears.

Because the `stale_entries` JSON, the `staleness_warnings` count, and
`requirements_stale_specs` are all populated inside that branch, every output format
follows without a second change.

## 3. Quoting is handled once, at the parse layer

`unquote_yaml_scalar` strips matched surrounding quotes, discards a comment following the
closing quote, and returns an error for an opening quote with no close — matching what
flow-style lists already did.

It is applied to block list items and to scalar values, so it covers `files:`,
`depends_on:`, `db_tables:` and every scalar at once; `status: "active"` had the same
defect. A value starting with `[` is passed through untouched so `parse_flow_string_list`
keeps ownership of its own unquoting.

Because `strip_yaml_comment` already declines to touch a quoted value, a `#` inside quotes
survives and a `#` after the closing quote is removed — the two cases that would otherwise
trade places.

`hash_cache::extract_frontmatter_files` is a deliberate second mini-parser that avoids a
circular dependency. It gets the same unquoting inline: without it the cache keys entries
on `"src/a.rs"`, a path no file exists at, so every run would see the spec as changed.

## 4. The remediation is bounded

`UNCOVERED_PATH_FLAG_LIMIT` (12) paths are spelled out; beyond that the message states how
many remain and suggests a covering prefix such as `--path src/`.

## Public API added

| Symbol | Module |
|---|---|
| `HashCache::has_baseline` | `hash_cache` |
| `ChangeClassification::reportable` | `hash_cache` |
| `ChangeClassification::baseline_known` | `hash_cache` |
| `ValidationResult::had_present_source` | `types` |
| `ValidationResult::documents_contract` | `types` |

## Deliberately not done

**#546 — a benign intra-project symlink aborts `check`/`coverage`/`score`/`generate`.** The
guard is a capability-confinement property: the coverage walk never follows a link, so a
link can never redirect discovery outside the retained root. Resolving the target and
re-checking containment reintroduces that escape and is TOCTOU-prone. The likely correct
fix is to skip the entry and disclose it, so coverage numbers stay honest about what was
excluded — a different change with its own coverage-accounting consequences.
