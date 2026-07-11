---
change: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
artifact: context
---

# Context

The YAML extractor emits dotted public paths such as `inputs.config`,
`outputs.status`, `permissions.contents`, and `jobs.trust`. The Public API table
parser currently captures only `\w+`, so the same documented rows are truncated
to their first segment and strict validation reports false undocumented exports.

CorvidLabs/trust is the release-blocking consumer. Its contract remains in
review until the locally built fix proves that all 30 extracted YAML symbols are
documented when the spec is temporarily promoted to active.
