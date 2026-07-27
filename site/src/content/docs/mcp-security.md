---
title: "MCP Security and Limits"
section: "Reference"
order: 5
---

The MCP server is read-only unless the operator starts it with `--allow-write`. Read operations use
retained directory capabilities and immutable bounded snapshots. Mutating tools always use the
retained server-root capability and reject per-call root overrides. Replacing a configured path
cannot redirect an active operation.

Every generic project file, selected config, and recognized manifest is opened no-follow and
non-blocking through a retained directory. Its path identity must still match the opened handle
before and after the bounded read. FIFOs, sockets/devices, links/reparse points, and regular-file
replacement races therefore fail without blocking or consuming replacement bytes, for both tools
and resources.

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
not found. Spec discovery is part of that trust decision: directory-walk failures, non-UTF-8 spec
names, unreadable specs, malformed or missing frontmatter, and invalid top-level `implements` or
`tracks` list shapes make the result inconclusive instead of producing a successful partial or
zero-reference result. The shared maintained `serde-saphyr` checked parser rejects duplicate keys
or malformed YAML anywhere, plus blank, null, scalar, mapping, mixed, non-positive, or overflowing
known fields. YAML comments and valid trailing commas remain supported. Nested extension mappings,
sequences, and block-scalar text named `implements` or `tracks` are not issue-reference fields. LF
and CRLF frontmatter delimiters are accepted equivalently.
Diagnostics contain only bounded, sanitized project-relative paths and content-free reason classes.
A completely inspected project with no issue references is reported directly without requiring
provider access.

## Manifest discovery

MCP snapshot and confinement discovery parse bounded Cargo manifests as real TOML, including
multiline workspace arrays and comments. Invalid TOML or malformed workspace shapes make the
operation inconclusive; partial member paths are not trusted. Gradle settings use the shared checked,
comment- and escape-aware parser for Groovy and Kotlin includes plus supported `projectDir`
overrides. Malformed Gradle discovery likewise fails gates instead of falling back to a partial or
empty success. Every present build/settings filename is preflighted before precedence selection,
including a lower-precedence shadowed variant, and remains identity-bound across its 4 MiB retained
read. Invoked unsupported inclusion APIs and indirect/conditional include or project-directory
mutations fail closed, while unrelated Gradle control flow remains compatible.

Cargo member declarations and Node workspace patterns also consume a deterministic expansion-work
budget before traversal. Normalized workspace nodes are deduplicated and completed discovery
results are reused, so repeated declarations cannot amplify cached parsing work.

`compute_coverage_checked` retains one project capability before configuration or zero-config
source detection. Explicitly configured source directories are parsed first and do not trigger
unrelated autodetection. Caller-selected spec mappings, every recognized manifest and nested
workspace probe, spec-module enumeration, and source traversal remain bound to that capability
through the checked-coverage root verification. Nested config/manifest parents must remain
reachable from the retained root, and selected-spec identities captured during inventory remain
authoritative during ownership parsing. The deterministic iterative snapshot shares limits of
8 MiB per selected spec or source file, 64 MiB of selected-spec/source bytes, 100,000 inventory
entries, and 256 path components. Invalid UTF-8 names/content, special entries, links/reparse
points, identity replacement, and exhausted limits make coverage inconclusive rather than yielding
a partial percentage.

Security regression instrumentation keeps separate checkpoints immediately after checked-coverage
root retention and after retained manifest discovery but before selected-spec/source traversal.
Gate callers propagate failures from either checkpoint. This guarantee is scoped to the
`compute_coverage_checked` operation: a command-wide immutable CLI analysis snapshot and generic
structured discovery outcomes remain assigned to later CLI/outcome/generation work outside
GitHub #414's MCP boundary. Hosted Windows runtime remains the acceptance authority for
junction/reparse behavior.

Cargo path authority comes only from semantic target, dependency, workspace-dependency,
target-specific dependency, patch, and replacement tables. An arbitrary metadata key named `path`
does not authorize snapshot input. Semantic manifest-relative paths may normalize parent components
across sibling crates, and confined Windows-native backslashes are normalized equivalently, when
the result remains beneath the configured server root. Drive-prefixed, UNC, rooted, traversal,
canonical, symlink, and Windows-junction escapes are rejected.

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
