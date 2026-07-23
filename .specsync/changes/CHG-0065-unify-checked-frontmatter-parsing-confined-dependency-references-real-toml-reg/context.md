---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: context
---

# Context

CHG-0065 groups issues #413, #419, #422, #436, and #444 because they share the same untrusted
input and verdict pipeline. Fixing any one command independently would preserve contradictory
parsing, path handling, and exit behavior elsewhere.

The issue boundaries are exact. #413 owns dual registry-shape support and fail-closed TOML with the
established local-registry diagnostic. #419 owns the `deps --require-coverage` gate—including
`101` and zero-source behavior—plus scalar dependency rejection, edge deduplication, and the
intentional allowance for valid unknown extension fields. #422 owns primary/legacy remote registry
ordering, explicit-token authentication, truthful zero-fetch handling, exit 1, and JSON. #436 owns
duplicate/type/shape/YAML diagnostics, flow/block dependency parsing, confinement, and module/path
identity. #444 owns local unresolved/malformed exit 1, raw diagnostic spelling, JSON, bounded
parallel timeout behavior, and the corrected local-dependencies heading.

At the base commit, `src/parser.rs` is a line scanner. Duplicate keys overwrite earlier values,
unknown and malformed lines are ignored, scalar and list shapes are conflated, invalid issue
numbers disappear, and flow-style dependency lists are not decoded as dependency edges. The public
`parse_frontmatter` return type cannot explain whether delimiters, YAML syntax, field type, field
shape, or value semantics failed. Gating consumers commonly interpret `None` as “skip this spec,”
which creates false-green graph and validation results.

Dependency parsing is also duplicated. `src/validator.rs` classifies a remote reference by the
presence of `/` and `@`, `src/deps.rs` extracts modules heuristically from path strings,
`src/commands/resolve.rs` joins local values directly to the project root, and `src/scoring.rs`
treats every dependency as a filesystem path. This lets absolute and parent-traversal references
escape the intended project namespace, makes malformed remote refs vanish, penalizes valid bare or
remote references, and gives check/deps/resolve different verdicts. Canonical module identity is
not cross-checked against the spec path, while registry-mapped custom paths require an explicit
exception to a simple directory-name rule.

`src/registry.rs` currently scans lines rather than parsing TOML. It understands only `[specs]`,
while `site/src/content/docs/cross-project-refs.md` documents `[[modules]]`; malformed TOML can
retain a non-empty apparent `name` and be accepted with mappings silently lost. The valid inert
5.0.1 stub compatibility introduced earlier must remain, but it applies only after successful TOML
parsing proves that no registry identity or mappings exist.

Remote resolution currently requests root `specsync-registry.toml`, even though `init-registry`
and the docs publish `.specsync/registry.toml`. It does not use `GITHUB_TOKEN`, fetches repositories
sequentially, ignores JSON output, and may print “All cross-project references verified” when every
fetch failed. Local resolution prints red crosses but exits zero. Dependency coverage is similarly
disconnected: the global `--require-coverage` flag reaches other gate commands but is not passed to
`deps`, including visualization modes.

The intended compatibility boundary is narrow:

- valid existing block-list frontmatter, leading BOMs, CRLF files, valid unique extension keys,
  and canonical `[specs]` registries continue to work;
- documented `[[modules]]` registries become supported, while generation remains canonical
  `[specs]`;
- bare modules resolve through a registry mapping before the conventional spec location;
- unknown frontmatter extensions remain allowed, but duplicate keys and malformed known fields do
  not;
- no network access occurs unless remote verification is requested; public repositories may be
  read anonymously and private repositories use explicit `GITHUB_TOKEN`; and
- changed warning/strict and unresolved-reference exit semantics are release-noted because CI that
  previously relied on a false green will begin failing correctly.

The work is sequenced so `types` and `parser` establish the shared contract before downstream
modules migrate. Registry/GitHub transport and graph/coverage work can then proceed in parallel;
validator/scoring, resolve, and CLI integration follow. MCP migration is last because `src/mcp.rs`
is also part of issue #414 security work. CHG-0065 must consume that retained-root confinement
rather than duplicating or weakening it.

Primary implementation and evidence locations are:

- `src/types.rs`, `src/parser.rs` for diagnostics and typed references;
- `src/registry.rs`, `src/github.rs` for real TOML and authenticated bounded content reads;
- `src/deps.rs`, `src/commands/deps.rs` for graph and coverage gates;
- `src/validator.rs`, `src/commands/check.rs`, and `src/scoring.rs` for check/score parity and
  module identity;
- `src/commands/resolve.rs` for typed outcomes, truthful summaries, caching, and bounded fanout;
- `src/cli.rs`, `src/main.rs`, and `src/commands/mod.rs` for exact formats and exit codes;
- `src/mcp.rs` for post-#414 parity; and
- `tests/integration/check.rs`, `tests/integration/commands.rs`, `tests/integration/config.rs`, and
  `tests/integration/mcp.rs` for end-to-end evidence.

The private `CorvidLabs/spec-sync-sandbox` testbed already contains
`drills/012-registry-parser-realities.sh`, which reproduces issue #413 against 5.2.0. This change
turns that drill into a passing assertion and adds separate parity, local-resolution, and
authenticated-remote drills. Sandbox evidence supplements repository tests; it does not replace
platform CI, strict coverage, trust verification, or independent review.

Out of scope are unrelated frontmatter schema extensions, a new dependency syntax, automatic
rewriting of user specs into a preferred YAML style, changing the emitted registry away from
`[specs]`, silently repairing malformed registries, or weakening remote verification to preserve
the old exit-zero behavior.
