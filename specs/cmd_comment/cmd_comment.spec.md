---
module: cmd_comment
version: 9
status: stable
files:
  - src/commands/comment.rs
db_tables: []
tracks: []
depends_on:
  - specs/change/change.spec.md
  - specs/commands/commands.spec.md
  - specs/comment/comment.spec.md
  - specs/github/github.spec.md
  - specs/ignore/ignore.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Comment

## Purpose

Implements the `specsync comment` command. Generates a spec-sync check summary as markdown and optionally posts it as a GitHub PR comment via `gh pr comment`.

**This is the single source of PR comment output for all spec-sync integrations.** Both the marketplace GitHub Action (`action.yml`, `comment: true`) and the project's own CI workflow (`.github/workflows/ci.yml`) invoke `specsync comment` (without `--pr`) to capture the markdown body, then post it via their respective GitHub API methods. This guarantees identical comment content regardless of invocation method.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_comment` | `root: &Path, pr: Option<u64>, _base: &str, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>` | `()` | Generate check summary; post as PR comment if `--pr N` is set, otherwise print to stdout |

## Invariants

7. Coverage uses checked manifest discovery; malformed Gradle settings produce an inconclusive
   stderr diagnostic and exit 1 before a misleading PR summary can be emitted.

## Behavioral Examples

### Scenario: Print to stdout

- **Given** `--pr` is not set
- **When** `cmd_comment` runs
- **Then** prints markdown summary to stdout

### Scenario: Post to PR

- **Given** `--pr 42` is set
- **When** `cmd_comment` runs
- **Then** posts comment on PR #42

### Scenario: Marketplace action captures stdout

- **Given** the marketplace action runs with `comment: true`
- **When** `specsync comment` is invoked without `--pr`
- **Then** the stdout output is identical to what the CI workflow captures via `cargo run -- comment`

### Scenario: Configured verification command emits output

- **Given** CI configures an SDD verification command that writes to stdout or stderr
- **When** `specsync comment` runs
- **Then** the command still executes and affects lifecycle status, but its child output is absent from the rendered markdown stream

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `gh` CLI not installed | Command fails with error |
| GitHub repo unresolvable | Exits 1 |
| Malformed Gradle settings prevent coverage discovery | Prints an explicit "Coverage inconclusive" error to stderr and exits 1 |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns` |
| comment | `render_check_comment` |
| github | `resolve_repo` |
| validator | `validate_spec`, `compute_coverage_checked` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync comment` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/change/change.spec.md`, `specs/ignore/ignore.spec.md`, `specs/types/types.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v6: fail closed when malformed Gradle/manifest discovery prevents trustworthy coverage |
| 2026-04-11 | Documented unified pipeline: marketplace action and CI both use `specsync comment` for identical PR comments |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting: Close final fail-closed review gaps in 5.0 lifecycle evidence and PR reporting |
| 2026-07-11 | CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re: Close final SpecSync 5.0 evidence, monorepo, bootstrap, reporting, and import review gaps |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-12 | CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i: Sever specsync check and comment from the trust layer: lifecycle state becomes informational and never affects exit status |
| 2026-08-14 | CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail: Specsync comment must exit with the verdict it prints, so a failing comment fails the CI step that posted it |
