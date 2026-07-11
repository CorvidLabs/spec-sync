---
change: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
artifact: testing
---

# Testing

## Requirement Evidence

- `REQ-parser-001`: focused unit coverage for dotted YAML paths, hyphens,
  ordinary identifiers, other extractor punctuation found by the grammar audit,
  first-symbol behavior, excluded subsections, empty delimiters, and malformed
  rows; integration coverage for an active GitHub Actions YAML contract under
  `check --force --strict`.
- `REQ-change-parser-001`: the same integration regression plus temporary
  CorvidLabs/trust promotion proves exact extractor-to-contract matching in a
  real consumer with all 30 YAML exports documented.

Run every configured Fledge lane, strict SpecSync self-validation, release
checks appropriate before a patch PR, and Augur. Dogfood the resulting local
binary against CorvidLabs/trust with its spec temporarily active, then restore
and verify the Trust worktree byte-for-byte.

## Results

- `fledge lanes run pre-commit`: passed.
- `fledge lanes run check`: passed.
- `fledge lanes run ci`: passed, including 1,529 unit tests, 188 integration
  tests, release build, dependency audit, strict spec validation, docs, and the
  VS Code extension package.
- `fledge lanes run repo`: passed.
- `specsync check --strict --require-coverage 100 --force`: 62/62 specs passed,
  zero warnings, 100% file and LOC coverage.
- `augur check --staged`: `REVIEW`, risk 41/100, confidence 59/100; no
  block verdict. Human review remains required before merge.
- Trust dogfood used the local `specsync 5.0.1` binary in a detached temporary
  worktree with `trust.spec.md` promoted to `active`: 30/30 exports documented,
  one spec passed, zero warnings, zero failures. The full Trust verification
  lane passed with Augur `proceed` at risk 35 and progressive provenance.
- The primary `/Users/leif/Development/_CorvidLabs/trust` checkout remained
  clean, and the temporary worktree was removed after verification.
