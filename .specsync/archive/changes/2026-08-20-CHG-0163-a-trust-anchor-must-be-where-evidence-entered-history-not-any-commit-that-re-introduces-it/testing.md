---
change: CHG-0163-a-trust-anchor-must-be-where-evidence-entered-history-not-any-commit-that-re-introduces-it
artifact: testing
---

# Testing

`an_archived_package_is_authenticated_only_by_where_its_evidence_entered_history` drives six
fixtures through a real repository with real commits.

| Fixture | Expectation | On `origin/main` |
|---|---|---|
| untouched archive | authenticated | authenticated |
| committed tamper, no relocation | refused | refused |
| tamper then `git mv` | refused | **anchored `2746d148`** |
| tamper and relocate in one commit | refused | **anchored `0e53b51a`** |
| forged reopen/re-archive, no rename | refused | **anchored `6360b7f4`** |
| honest relocation, byte-identical | authenticated | authenticated ← control |

The honest-relocation control is load-bearing. Without it, "refuse every relocated archive" turns
the test green — and breaks the slug-only migration this fix exists to enable.

## Corpus regression

One representative per risk class against a 161-row baseline captured before the fix: legacy
(44), stage A+B (19), stage B only (90), and pre-existing corrupt (7). All four match.

## Sandbox

Drill `069-anchor-reintroduction` pins the same property against a real archived corpus, which
the Rust suite structurally cannot: it needs a real repository, a real addition, and a real
`R100` rename. It was originally written as `069-archive-rename-launders-tampering` and renamed
here, because a drill pinning only the rename would go green on a fix that closed only the
rename.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-085 | Three laundering shapes are refused here and each resolves an anchor against a separate checkout of `origin/main`, so the test discriminates on all three rather than on the one first reported. The honest-relocation control passes on both binaries, proving the refusals were not obtained by refusing relocation generally. Corpus regression is sampled per risk class against a pre-fix baseline, with the stage-B-only majority — the class that breaks if the bound is too tight — explicitly covered |
