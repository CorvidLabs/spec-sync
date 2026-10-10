---
change: split-validator-validation-and-retained-coverage-into-focused-private-modules
artifact: research
---

# Research

Compared validator 6416 lines with manifest 5465. Validator is the cleaner next boundary because its full ordered validation core is 670 lines, schema/policy/mapping helpers are separable, and retained coverage already passes state explicitly. Manifest's interleaved Gradle grammar and guards are more coupled. Public GitHub API at 18:54 UTC reports only PR 800 open, with no validator overlap. Existing main full-unit baseline is 2338 passing and 192 failing in this sandbox; all failure names match the earlier MCP refactor comparison. Historical repository claims are not fresh evidence.
