---
id: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
state: archived
type: bug_fix
base_commit: 88d73e1a8208f25035ee727f2326f871af504223
---

# The config refusal must guard both loaders, because load_config is a second door through which rules, compact and rehash reported success over configuration they never read

## Intent

The config refusal must guard both loaders, because load_config is a second door through which rules, compact and rehash reported success over configuration they never read

## Affected Canonical Specs

- `config`
- `cmd_wizard`
- `cmd_init_registry`

## Acceptance Criteria

- A project whose config file exists but cannot be read is refused by commands that load config directly, not only by those that go through the discovery path. rules, compact and rehash all exit non-zero, and rehash writes no hash cache. A project with a valid config is unchanged in every one of them. The repair paths that must keep working over a broken config ask for the permissive loader by name, so a caller added later gets the guard unless it deliberately opts out.

## No-spec Rationale

Not applicable
