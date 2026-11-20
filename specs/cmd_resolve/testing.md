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

## Symbolic links (REQ-cmd-resolve-002)

Every fixture is one temporary directory with the project and an `outside` directory in it, so no link in a test reaches a real file. The link tests are Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A path this module writes leaves the project through a link, dangles, or is not a regular file | Refused with an error naming the path; nothing outside the project changes | `src/commands/resolve.rs`: `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory`. No test drives a live `resolve --remote`, which needs the network |

## Bounded reads (REQ-cmd-resolve-003)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |

## The remote spec cache outside the work tree (REQ-cmd-resolve-002)

| Case | Required behavior | Test |
|------|-------------------|------|
| A forged entry committed under `.specsync-cache/remote-specs/` | Never returned | `src/commands/resolve.rs`: `spec_cache_never_returns_an_entry_the_work_tree_holds` |
| A link or a dangling link at an entry in the local state folder, a linked `.specsync-cache` | Never read through; storing replaces the link and creates nothing at its target | `src/commands/resolve.rs`: `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory` |
