---
change: CHG-0012-correct-specsync-5-0-documentation-cli-help-and-hub-deep-links
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-cli-args-003 | Root and `new --help` smoke checks confirm canonical configuration, output formats, and companion wording; the complete 1,527-unit and 187-integration suite proves argument parsing and command behavior remain unchanged |

Documentation evidence includes 23 Bun tests, zero Astro diagnostics, a successful
39-page Astro build, strict validation of 62 specs at 100% file and LOC coverage,
Rust formatting, and page-specific redirect inspection.
