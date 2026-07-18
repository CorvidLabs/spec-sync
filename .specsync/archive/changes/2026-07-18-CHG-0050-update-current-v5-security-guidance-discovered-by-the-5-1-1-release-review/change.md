---
id: CHG-0050-update-current-v5-security-guidance-discovered-by-the-5-1-1-release-review
state: archived
type: documentation
base_commit: fd2eb796b4026326e1390fc9709d341d6edeb0c5
---

# Update current v5 security guidance discovered by the 5.1.1 release review

## Intent

Update current v5 security guidance discovered by the 5.1.1 release review

## Affected Canonical Specs

- None

## Acceptance Criteria

- SECURITY.md recommends CorvidLabs/spec-sync@v5 for current-major CI usage, the release validator scans SECURITY.md YAML examples so stale or moving pins fail

## No-spec Rationale

This corrects a user-facing Action pin from the retired v4 line to v5 without changing canonical product behavior or public APIs.
