---
spec: cli.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Add shell completion generation subcommand (`specsync completions bash/zsh/fish`)
- [ ] Add `--quiet` flag to suppress non-error output
- [ ] Add `--color never/always/auto` flag for explicit color control

## Done

- [x] Implement the full subcommand surface (check, coverage, generate, init, score, watch, mcp, add-spec, scaffold, init-registry, resolve, diff, hooks, compact, archive-tasks, view, merge, issues, new, wizard, deps, import, stale, report, comment, rules, changelog, rehash, migrate, lifecycle)
- [x] Add `--json` output mode (shorthand for `--format json`) and `--format text/json/markdown`
- [x] Add `--strict`, `--require-coverage`, `--enforcement`, `--exclude-status`, `--only-status` global flags
- [x] Add `--root` flag for non-cwd project roots
- [x] Make `check` the default subcommand when none is specified
- [x] Remove embedded provider/model generation flags and preserve deterministic agent integrations
- [x] Wrap `run()` in `catch_unwind` so panics surface a friendly bug-report message
- [x] Block inherited verification recursion before dispatching `change` or `lifecycle` subcommands
- [x] Forward explicit MCP write authorization and fail closed on server-root startup errors
- [x] Preserve requested roots for MCP and coverage-gating retained-capability traversal
- [x] Keep generate writes bound to the retained root after checked coverage returns

## Gaps

- No shell completion support
- No `--quiet` flag for CI pipelines that only want exit codes

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
