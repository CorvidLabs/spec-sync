---
id: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
state: archived
type: bug_fix
base_commit: 1b3974e9a025f37ce712d4ad627e99f780d6f207
---

# Keep writes, reads and caches a repository can steer inside its Git work tree

## Intent

Keep writes, reads and caches a repository can steer inside its Git work tree

## Affected Canonical Specs

- `agents`
- `archive`
- `change`
- `changelog`
- `cmd_check`
- `cmd_generate`
- `cmd_import`
- `cmd_init_registry`
- `cmd_init`
- `cmd_issues`
- `cmd_lifecycle`
- `cmd_merge`
- `cmd_migrate`
- `cmd_view`
- `commands`
- `compact`
- `config`
- `deps`
- `generator`
- `git_utils`
- `hash_cache`
- `hooks`
- `mcp`
- `merge`
- `rehash`
- `util`
- `validator`

## Acceptance Criteria

- Every change-workflow, backup, registry, init, lifecycle, merge and generator write goes through the confined project helpers: a committed change.md or artifact link out, a dangling artifact link, a linked .specsync/changes, a linked .specsync/backup-fix and a linked .specsync under init-registry are refused with the path named, and nothing outside is written. Inside a Git work tree the helpers confine to the work tree's top folder, so a monorepo subproject's links that stay in the repository work; absolute links are refused as absolute links; a path into the Git directory (a .git component in any case, trailing dots or spaces stripped, or a link whose canonical target lies under git rev-parse --absolute-git-dir or a .git component) is refused as leading into the Git directory; a write opens without blocking and checks the handle is a regular file before truncating. A .specsync/hashes.json Git tracks is neither read nor saved: check validates every spec with one warning naming git rm --cached, rehash exits 1, and prune drops keys that leave the project. specs_dir, source_dirs and schema_dir must be relative paths inside the project without ..; spec discovery walks the confined handle within 100,000 entries and 256 levels; changelog skips a spec blob over 8 MiB at a ref with one warning; import --from-dir reads within the named directory with the 8 MiB bound. A source scan fails on any new direct write, rename or removal outside the helpers, and the read scan also counts fs::copy, OpenOptions reads, BufReader and io::copy. The new tests fail on the base with only the tests added and pass with the fix

## No-spec Rationale

Not applicable
