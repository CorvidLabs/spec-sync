---
id: align-public-documentation-and-the-release-runbook-with-the-shipped-specsync-6-0-0
state: archived
type: documentation
base_commit: 38354c51ad3a4ba2408b8daf906dd1a9a0d5659f
---

# Align public documentation and the release runbook with the shipped SpecSync 6.0.0

## Intent

Align public documentation and the release runbook with the shipped SpecSync 6.0.0

## Affected Canonical Specs

- None

## Acceptance Criteria

- README, site docs, MIGRATION, ADOPTING, SECURITY, CONTRIBUTING, SCOPE, AGENTS, action metadata text, example READMEs and ci-confidence describe the shipped 6.0.0 binary and workflow with no candidate-window wording, retired CHG identities, rejected flags, or Windows support claims; the import caveat (#416) and the pre-adopt v1 guidance (#674) are present; docs/RELEASING.md gives a maintainer the exact qualify, promote, crates.io and Homebrew sequence; site lint, tests and build pass; release-version and runtime-pin validators pass; the example scripts still pass with the 6.0 binary

## No-spec Rationale

Documentation, runbook, Action metadata text, and lane comments only; no canonical spec contract, command grammar, or runtime behaviour changes
