---
id: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
state: archived
type: bug_fix
base_commit: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
---

# Walk configured folders only through the confined walk, and keep the read cache in the user cache folder

## Intent

Walk configured folders only through the confined walk, and keep the read cache in the user cache folder

## Affected Canonical Specs

- `util`
- `change`
- `validator`
- `config`
- `schema`
- `commands`
- `cmd_scaffold`
- `registry`
- `cmd_wizard`
- `mcp`
- `cmd_check`

## Acceptance Criteria

- Every walk or listing of a configured folder (specs_dir, source_dirs, schema_dir) goes through util's confined walk (find_project_files, read_project_dir), so a committed link that leaves the project at the folder or on the way to it is refused with the path instead of walked: change adopt's requirement proposals, the duplicate requirement ID check, init-registry's registry, the MCP issues walk, the wizard's source walk, the schema snapshot and check's schema inputs, scaffold's case-collision listing, a missing file's suggestion, and source auto-detection, which skips linked folders. The change read cache (archived history verdicts) lives in specsync/repositories/<key> in the user's cache folder, keyed by the canonical Git common directory so worktrees still share it, never in the Git directory, and is refused like the other state when the cache folder lies inside the project. The new tests fail on 4516f355 with only the tests added and pass with the fix

## No-spec Rationale

Not applicable
