---
id: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
state: archived
type: feature
base_commit: eb91993ba5289e317dcfc22156c6202b85273c98
---

# Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts

## Intent

Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts

## Affected Canonical Specs

- `commands`
- `config`
- `types`
- `validator`
- `comment`
- `output`

## Acceptance Criteria

- Draft specs may map safe normalized missing source paths without strict validation errors
- Planned mappings appear separately in text JSON Markdown and GitHub output and never count toward current file or LOC coverage
- Active specs and configurations with require_draft_files enabled continue to reject missing source paths
- Existing source validation generation issue reporting and coverage behavior remain unchanged for present files
- Incremental validation detects duplicate ownership against both changed and cached specs
- Canonical config commands comment output types and validator contracts document every new field signature and behavior

## No-spec Rationale

Not applicable
