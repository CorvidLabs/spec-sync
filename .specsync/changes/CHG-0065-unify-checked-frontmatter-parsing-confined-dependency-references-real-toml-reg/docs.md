---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: docs
---

# Docs

## Issue-to-documentation binding

| Issue | Required public correction |
|-------|----------------------------|
| #413 | Show and support `[[modules]]` and `[specs]`, identify `[specs]` as emitted form, and document malformed TOML's established `failed to parse local registry … while resolving …` failure. |
| #419 | Document active `deps --require-coverage` behavior for below-threshold, `101`, and zero-source cases; list-only `depends_on`; deduplicated edges; intentionally allowed valid unknown extensions. |
| #422 | Document primary `.specsync/registry.toml`, 404-only root fallback, `GITHUB_TOKEN`, exit 1 on zero/failing fetches, and honored JSON. |
| #436 | Document duplicate rejection, flow/block lists, path confinement, module/path identity, and diagnostics for malformed YAML or wrong types/shapes. |
| #444 | Document exit 1 and raw diagnostics for unresolved/malformed local refs, honored JSON, bounded parallel remote timeouts, and corrected “Local dependencies” output. |

## Public documentation changes

Update `site/src/content/docs/spec-format.md` to define frontmatter as checked YAML rather than a
best-effort line format. Document:

- required scalar and list shapes for every known field;
- equivalent block and flow list examples for `depends_on`;
- duplicate-key rejection, including duplicate extension keys;
- continued support for valid unique unknown extension fields;
- accepted version values and typed `implements`/`tracks` lists;
- the three dependency forms: bare local module, explicit project-relative `.spec.md` path, and
  `owner/repository@module`; and
- portable path confinement: no absolute, drive-relative, UNC/extended, parent traversal,
  backslash, or symlink/junction escape references.

Explain that canonical `specs/<module>/<module>.spec.md` paths must agree with `module:`, while a
registry key authorizes a custom mapped spec location.

## Registry and remote-resolution guide

Correct `site/src/content/docs/cross-project-refs.md` so the generated and accepted formats cannot
contradict one another:

- show canonical `[registry]` plus `[specs]` as the format emitted by `init-registry`;
- retain a compatibility example for accepted `[[modules]]` records;
- state that conflicting duplicate mappings and malformed TOML fail closed;
- preserve the valid inert 5.0.1 stub explanation without implying malformed files are inert;
- state that every registry path is project-relative and confined;
- identify `.specsync/registry.toml` as the primary remote location and root
  `specsync-registry.toml` as a legacy fallback attempted only after a primary 404;
- document optional `GITHUB_TOKEN` authorization for private repositories and token-redaction
  guarantees;
- document the bounded operation/invocation timeout and bounded repository concurrency; and
- remove any example suggesting a registry/provider failure is an advisory success.

Add examples for a mapped non-conventional spec, a missing remote module, all-provider failure, and
private-repository verification. Examples must show the corresponding exit status and must never
embed a credential.

## CLI reference and structured output

Update `site/src/content/docs/cli.md` for `deps` and `resolve`:

- `deps --require-coverage N` gates text, JSON, Mermaid, and DOT output through checked coverage;
- thresholds outside `0..=100` are usage errors;
- zero-source and malformed-discovery coverage are inconclusive failures, not vacuous success;
- local missing, malformed, or escaping dependencies fail independent of `--strict`;
- `resolve --remote`/`--verify` fail when any requested registry, module, or spec cannot be
  verified;
- `--strict` promotes advisory compatibility findings such as non-bidirectional refs;
- “no drift” is printed only after at least one registry and every requested reference are
  successfully verified; and
- remote repositories are fetched with bounded parallelism and versioned warm/cold-equivalent
  caching.

Publish the common exit-code contract:

| Exit | Documentation text |
|------|--------------------|
| `0` | Trustworthy success or an advisory run with no failed requested gate |
| `1` | Finding, unresolved dependency, inconclusive provider/coverage result, or failed gate |
| `2` | Invalid command usage |

Extend the JSON section with representative `deps` and `resolve` documents. Both include
`valid`, `gate_passed`, diagnostics, and per-reference raw values; `deps` includes coverage when a
coverage gate is requested, and `resolve` includes remote/cache outcome provenance. Failing JSON
examples remain one parseable document with no ANSI or human preamble.

## Canonical specs and release notes

Update the canonical specs and present companions for `types`, `parser`, `registry`, `github`,
`deps`, `cmd_deps`, `validator`, `scoring`, `cmd_resolve`, `cli`, `commands`, and `mcp`. Each changed
spec increments `version`, records the new public or behavioral contract, maps requirement IDs to
tests, and adds a dated change-log row. MCP documentation lands only after the #414 retained-root
security implementation is integrated.

Add an `[Unreleased]` compatibility/security entry to `CHANGELOG.md` that calls out:

- malformed or duplicate frontmatter that previously passed now fails;
- dependency traversal and malformed references are rejected consistently;
- `deps --require-coverage`, local resolve, remote provider failures, and strict compatibility
  findings now return meaningful nonzero statuses;
- JSON now exposes explicit `valid` and `gate_passed` verdicts;
- private remote reads use `GITHUB_TOKEN`; and
- registries now parse real TOML and accept both documented and canonical forms while generation
  continues to emit `[specs]`.

The release note must frame newly failing CI as correction of prior false-green behavior and give
the remedy: fix malformed frontmatter/references, add coverage, commit a valid registry, configure
authentication, or remove a gate that was not intended to block.

## Evidence and publication checks

- Build the Astro documentation site and fail on broken links or malformed examples.
- Execute every documented shell/JSON example against the release candidate where practical.
- Verify documented registry examples round-trip through the real parser.
- Validate every JSON example with a JSON parser.
- Ensure no token, private repository response body, temporary path, or sandbox credential enters
  committed docs or lifecycle evidence.
- Record the private sandbox commit and redacted drill results in `testing.md`; keep private testbed
  implementation details out of public docs beyond the supported behavior they prove.
