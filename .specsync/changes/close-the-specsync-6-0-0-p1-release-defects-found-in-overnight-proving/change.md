---
id: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
state: implementing
type: bug_fix
base_commit: 0d0251bf62b30696a710a2a421becac02405539b
---

# Close the SpecSync 6.0.0 P1 release defects found in overnight proving

## Intent

Close the SpecSync 6.0.0 P1 release defects found in overnight proving

## Affected Canonical Specs

- `change`
- `cli_args`
- `cmd_check`
- `cmd_generate`
- `cmd_new`
- `cmd_scaffold`
- `config`
- `generator`
- `git_utils`
- `cli`
- `parser`
- `mcp`

## Acceptance Criteria

- cargo publish --dry-run compiles because lifecycle limits are bundled under src/; specsync new emits all seven init-required sections and refuses reserved names; malformed JSON config exits 1 on rules/rehash/compact/deps/archive-tasks/view; git children drop GITHUB_TOKEN and GIT_DIR and pin GIT_CEILING_DIRECTORIES; deleting or emptying verification-attempts.json refuses adopted finalization; CRLF definition payloads hash as LF; fenced Public API examples are not documented exports and check --fix ignores fenced ### headings; check --spec NAME parses; generate exits 1 when unspecced modules have no files; scaffold --dir cannot escape the project root; README and site quick start succeed without --strict on a stub scaffold.

## No-spec Rationale

Not applicable
