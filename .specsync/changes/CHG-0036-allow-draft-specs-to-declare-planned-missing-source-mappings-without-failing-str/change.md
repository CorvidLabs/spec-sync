---
id: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
state: accepted
type: feature
base_commit: a20ce51fbcc48ead4cfd550cc55a24ae48391075
---

# Allow draft specs to declare planned missing source mappings without failing strict validation or changing coverage denominators

## Intent

Allow draft specs to declare planned missing source mappings without failing strict validation or changing coverage denominators

## Affected Canonical Specs

- `config`
- `types`
- `validator`
- `commands`

## Acceptance Criteria

- By default, missing files mapped by draft specs produce explicit planned-mapping notices while strict validation passes.
- Planned missing paths never enter current file or LOC coverage denominators, while real mapped files retain structural, safety, readability, and ownership validation.
- Changing the same spec to active makes a missing mapping a strict error, and creating the planned file transitions it into normal mapping and coverage without configuration changes.
- The default-false require_draft_files setting round-trips through TOML and legacy JSON and makes missing draft mappings fail for teams that opt in.
- Draft-only, mixed draft and active, activation, file-creation, duplicate ownership, unsafe-path, configuration, native, and hosted regressions pass.

## No-spec Rationale

Not applicable
