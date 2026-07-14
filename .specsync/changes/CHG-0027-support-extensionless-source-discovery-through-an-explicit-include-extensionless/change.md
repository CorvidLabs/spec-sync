---
id: CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless
state: verifying
type: feature
base_commit: c98d29810f78abcdd6a2fec9b137667d3ab2fc5b
---

# Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, and LOC coverage regressions for extensionless-only and mixed projects

## Intent

Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, and LOC coverage regressions for extensionless-only and mixed projects

## Affected Canonical Specs

- `config`
- `validator`

## Acceptance Criteria

- Configuration accepts include_extensionless = true while omitted or false include_extensionless and omitted or empty source_extensions retain existing default discovery.
- An extensionless-only project discovers and maps a real source file with strict 100 percent file and LOC coverage over non-zero totals.
- A mixed project discovers both configured suffixed and extensionless files with strict 100 percent file and LOC coverage over non-zero totals.
- Parser, serialization, shared scanning, CLI integration, public documentation, canonical semantic deltas, and full native gates pass.

## No-spec Rationale

Not applicable
