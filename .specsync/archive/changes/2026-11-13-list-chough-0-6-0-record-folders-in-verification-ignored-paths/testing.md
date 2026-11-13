---
change: list-chough-0-6-0-record-folders-in-verification-ignored-paths
artifact: testing
---

# Testing

- `specsync check` passes with the edited policy.
- `chough doctor` reports no `records_in_verification` finding.
- `strict_verification_commands` for `.specsync/sdd.json` (strict path) run during
  `specsync change check`.
