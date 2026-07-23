---
title: "MCP Security and Limits"
section: "Reference"
order: 5
---

The MCP server is read-only unless the operator starts it with `--allow-write`. Read operations use
retained directory capabilities and immutable bounded snapshots. Mutating tools always use the
retained server-root capability and reject per-call root overrides. Replacing a configured path
cannot redirect an active operation.

Absolute outside roots are rejected before filesystem probing. Configuration, manifests, dependency
references, module definitions, spec mappings, source roots, generated destinations, nested symlinks,
and Windows junctions must remain beneath the configured server root. Explicitly configured source
roots remain in scope even if their names are normally ignored.

## Resource limits

| Resource | Limit |
|---|---:|
| JSON-RPC request | 1 MiB |
| JSON-RPC response | 1 MiB |
| Serialized request ID | 4 KiB |
| One project or configuration file | 8 MiB |
| Project/configuration bytes per operation | 64 MiB |
| Generated specs per call | 1,000 |
| Generated output per call | 64 MiB |
| Globally deduplicated issue IDs per invocation | 100 |
| One GitHub REST issue operation | 10 seconds |
| Authentication, repository preflight, and complete issue batch | 30 seconds |

Oversized, malformed, or out-of-root inputs fail closed. JSON-RPC envelopes and resource arguments
are validated before dispatch; invalid envelopes return `-32600`, while invalid resource or tool
arguments return `-32602`. Notifications never produce responses and cannot invoke mutations.

## GitHub issue verification

`specsync_issues` requires an explicit `github.repo` and `GITHUB_TOKEN` in MCP mode. The server
excludes `.git` from its project snapshot and never discovers an MCP repository from
project-controlled Git metadata. Issue reads, listings, and verification use in-process GitHub REST
requests and never launch a `gh` provider process. The `gh` CLI is reserved for the explicit
issue-creation write path outside read-only verification.

Repository access changes, authentication failures, transport errors, timeouts, and malformed REST
responses are inconclusive errors. Only a confirmed accessible-repository absence is classified as
not found. A project with no issue references is reported directly without requiring repository or
provider resolution.

## Manifest discovery

MCP snapshot and confinement discovery parse bounded Cargo manifests as real TOML, including
multiline workspace arrays and comments. Invalid TOML or malformed workspace shapes make the
operation inconclusive; partial member paths are not trusted. Gradle settings use the shared checked,
comment- and escape-aware parser for Groovy and Kotlin includes plus supported `projectDir`
overrides. Malformed Gradle discovery likewise fails gates instead of falling back to a partial or
empty success.

Cargo manifest-relative paths may normalize `..` across sibling crates when the normalized result
remains beneath the configured server root. Lexical or canonical escapes, including escapes through
symlinks or Windows junctions, are rejected.

## Generation and scoring

Generation fails on destination collisions or incomplete writes. Multi-file publication is
transactional: transaction-owned staged files are identity-bound, and rollback preserves
replacements at public destination and staging paths. A failed batch may leave an empty parent
directory that it created; SpecSync deliberately does not claim ownership after the non-atomic
create/open interval.

On Windows, transaction cleanup consumes the final quarantine directory capability before
name-based removal. This keeps init, generation, and collision rollback from failing with directory
sharing violations without weakening the quarantine identity checks.

The boundary protects MCP callers and project-controlled paths. It does not protect against a
same-user process already authorized to mutate the server root racing private transaction names.

Because Git contents are excluded from MCP snapshots, score output marks Git freshness unavailable
and conservatively withholds those five points.
