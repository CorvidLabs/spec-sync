---
spec: cmd_changelog.spec.md
---

## Tasks

- [ ] Add an end-to-end CLI test that runs `changelog FROM..TO` against a throwaway git repo and asserts the rendered output per format (the delegate is unit-tested; the wrapper's format dispatch is not)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified format dispatch: Json→format_json, Markdown→format_markdown, Text/Github/Table/Csv→format_text
- [x] Confirmed range parsing + report generation are covered by `changelog` inline tests (`test_parse_range_*`, `test_generate_changelog_*`, `test_format_*`)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
