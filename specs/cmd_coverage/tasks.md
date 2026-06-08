---
spec: cmd_coverage.spec.md
---

## Tasks

- [ ] Consider loading `IgnoreRules` from disk (like `check`/`comment`) instead of `IgnoreRules::default()`, or document why coverage intentionally ignores `.specsyncignore`

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] End-to-end CLI behavior is covered by integration fixtures (full/partial coverage, `--require-coverage`, `--strict`, JSON via MCP) — see testing.md
- [x] Verified JSON output keys and the two-decimal rounding / zero-denominator-as-100% behavior against the source
- [x] Verified exit-code delegation to `exit_with_status` for the non-JSON path

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
