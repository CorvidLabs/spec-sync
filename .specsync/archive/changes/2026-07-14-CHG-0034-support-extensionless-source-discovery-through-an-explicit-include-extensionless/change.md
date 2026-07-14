---
id: CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless
state: archived
type: feature
base_commit: 3b5d96806bd6fbc366f8bc22d6e61dd25b16af34
---

# Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, LOC coverage, and wizard regressions for extensionless-only and mixed projects

## Intent

Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, LOC coverage, and wizard regressions for extensionless-only and mixed projects

## Affected Canonical Specs

- `config`
- `validator`

## Acceptance Criteria

- Configuration accepts include_extensionless = true while omitted or false include_extensionless and omitted or empty source_extensions retain existing default discovery.
- An extensionless-only project discovers and maps a real source file with strict 100 percent file and LOC coverage over non-zero totals.
- A mixed project discovers both configured suffixed and extensionless files with strict 100 percent file and LOC coverage over non-zero totals.
- Wizard auto-detection excludes matching directory entries while retaining matching regular source files.
- Parser, serialization, shared scanning, CLI integration, public documentation, canonical semantic deltas, and full native gates pass.

## No-spec Rationale

Not applicable
