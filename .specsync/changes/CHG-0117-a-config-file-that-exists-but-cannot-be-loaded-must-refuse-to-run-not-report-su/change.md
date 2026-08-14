---
id: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
state: implementing
type: bug_fix
base_commit: a38c3554152b32867daf5df740c530d40aa221b5
---

# A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults

## Intent

A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults

## Affected Canonical Specs

- `types`
- `config`
- `validator`
- `commands`

## Acceptance Criteria

- A project whose config file exists but cannot be parsed or read is refused with an error naming the file, rather than silently falling back to built-in defaults and reporting that every configured rule passed. A project with a valid config continues to enforce exactly what it configures. A project with no config file at all continues to run on the built-in defaults and is unaffected. The refusal reaches every command that reads specs, not only the validating ones.

## No-spec Rationale

Not applicable
