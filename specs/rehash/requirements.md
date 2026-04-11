---
spec: rehash.spec.md
---

## User Stories

- As a developer, I want to regenerate my hash cache after switching branches so that `specsync check` works without `--force`
- As a CI operator, I want clear exit codes so that cache failures are actionable

## Acceptance Criteria

- `cmd_rehash` rebuilds hashes.json from scratch for all discovered specs
- Exits with code 1 on save failure with a clear error message
- Prints the number of specs hashed on success
