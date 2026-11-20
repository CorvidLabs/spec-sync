---
id: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths
state: implementing
type: bug_fix
base_commit: 6d155ce583e44b2411db84ec2ed307b1ed62bbdb
---

# Route and mark the listing and spawn sites the 6.4.1 scans found outside the hardening change's paths

## Intent

Route and mark the listing and spawn sites the 6.4.1 scans found outside the hardening change's paths

## Affected Canonical Specs

- `agents`
- `cmd_comment`
- `cmd_import`
- `cmd_merge`
- `cmd_migrate`
- `config`
- `generator`
- `github`
- `importer`
- `mcp`
- `watch`
- `change`
- `validator`

## Acceptance Criteria

- The read, write, Git and spawn scans of REQ-util-008 pass with every site in these 17 files routed through the confined listing (REQ-util-010) or marked with its reason; scaffold --template reads through read_typed_file (REQ-util-011, REQ-cmd-scaffold-009); change audit covers every changed path; no requirement text changes.

## No-spec Rationale

No requirement text changes: these files carry the approved REQ-util-008, REQ-util-010, REQ-util-011 and REQ-cmd-scaffold-009 of close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first, as scan markers, confined listings and typed-path reads
