---
spec: cmd_resolve.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/resolve.rs` | cargo test commands::resolve:: | `test_find_consumed_exports_parses_table`, `test_find_consumed_exports_skips_header_row`, `test_spec_cache_roundtrip`, `test_spec_cache_miss`, `test_spec_cache_expired`, `test_cache_path_sanitizes_slashes`, `test_verify_detects_deprecated_status` |

> These are helper-level unit tests (`find_consumed_exports`, `SpecCache`, `RemoteSpec` status). The `cmd_resolve` / `verify_remote_specs` network orchestration has no automated coverage yet — verify Behavioral Verification rows manually.

## Coverage Gaps

- Integration gap: add a fixture for "All local deps resolve" before changing user-visible CLI output, generated files, or error handling in cmd_resolve.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| All local deps resolve | all `depends_on` point to existing files | `cmd_resolve(root, false, false, 3600)` runs | prints green checkmarks for each resolved dependency |
| Remote registry check | a spec with `depends_on: ["corvid-labs/algochat@auth"]` | `cmd_resolve(root, true, false, 3600)` runs | fetches `specsync-registry.toml` from `corvid-labs/algochat` |
| Verify detects deprecated remote spec | a cross-project ref to `remote-repo@parser` | `cmd_resolve(root, true, true, 3600)` runs | prints `DRIFT remote-repo@parser: remote spec status is "deprecated"` |
| Verify detects missing export | local spec consumes `parse_ast` from `remote-repo@parser` | `cmd_resolve(root, true, true, 3600)` runs | prints `DRIFT ... but export 'parse_ast' no longer exists in remote spec` |
| Verify warns on non-bidirectional dependency | local spec depends on `remote-repo@parser` | `cmd_resolve(root, true, true, 3600)` runs | prints `WARN ... but remote spec does not reference <our-repo>` |
| Cache avoids redundant fetches | a prior `--verify` run cached remote spec content | `cmd_resolve(root, true, true, 3600)` runs within the TTL window | reads from `.specsync-cache/remote-specs/` instead of fetching |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Local dep missing | Warning printed | Keep or add a focused assertion before changing this behavior |
| Remote registry fetch fails | Warning, continues | Keep or add a focused assertion before changing this behavior |
| Remote spec fetch fails | Warning, continues | Keep or add a focused assertion before changing this behavior |
| Remote spec unparseable | Warning, continues | Keep or add a focused assertion before changing this behavior |
| Remote spec deprecated/removed | DRIFT error, exit 1 | Keep or add a focused assertion before changing this behavior |
| Consumed export missing from remote | DRIFT error, exit 1 | Keep or add a focused assertion before changing this behavior |
| Non-bidirectional dependency | Warning only | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- resolve --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/resolve.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
