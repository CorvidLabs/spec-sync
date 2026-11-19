---
id: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule
state: implementing
type: bug_fix
base_commit: 2cc0897fc00eee7db2b07f5134ba05639af39018
---

# A path under an exclude_dirs entry is not source for the verification_ignored_paths overlap rule

## Intent

a path under an exclude_dirs entry is not source for the verification_ignored_paths overlap rule

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- (1) With code at the root (source_dirs = ["."]) and exclude_dirs = [".chough"], verification_ignored_paths entries .chough/sessions/, .chough/archive/ and .chough/bugs/ are accepted, and verification ignores those folders. (2) An entry whose excluded component differs from the exclude_dirs name in case, or whose excluded name lies above the source_dirs root it is under, is still refused for overlapping source_dirs. (3) Every other overlap rule still refuses, even for an entry under an excluded name: .specsync/, the specs tree, schema_dir, meaningful_paths, spec files:, depends_on, Consumed By, the policy files, the principles file and the templates.

## No-spec Rationale

Not applicable
