## ADDED

### REQUIREMENT REQ-github-005

Remote registry and spec retrieval SHALL use one authenticated, bounded GitHub content transport
that distinguishes absence from authentication, provider, and network failures.

Acceptance Criteria

- Remote resolution requests `.specsync/registry.toml` first and requests legacy root
  `specsync-registry.toml` only after a confirmed primary-path 404.
- A 401, 403, timeout, malformed response, body-limit violation, rate limit, or transport failure
  does not trigger legacy fallback and remains an inconclusive error.
- When `GITHUB_TOKEN` is set, registry and spec-content requests send it as a GitHub API bearer
  token; public repositories remain readable anonymously where GitHub permits.
- Tokens and authorization headers are redacted from every error, debug, cache, and structured
  output path.
- Registry and spec bodies have deterministic byte limits; repository fetches use bounded
  concurrency under one invocation deadline rather than a sequential timeout per repository.
- Transport failures retain their original category through text, JSON, cache, and aggregate
  reporting; deadline expiry is never rewritten as connection refusal.
- Registry-provided spec paths are validated before constructing a request or cache path.
- Successfully fetched registries use the shared TOML parser, and successfully fetched specs use
  checked frontmatter parsing.
- Cache hits and misses have equivalent authentication, parsing, validation, and verdict semantics.

## MODIFIED

### SPEC SECTION Invariants

1. GitHub issue reads, lists, and verification remain in-process REST operations; only explicit
   drift-issue creation may invoke `gh`.
2. The shared content transport validates repository identity and confined registry-provided paths
   before URL or cache-path construction.
3. Remote registry lookup requests `.specsync/registry.toml` first and falls back to root
   `specsync-registry.toml` only after a confirmed 404.
4. Authentication, authorization, rate-limit, timeout, malformed-response, body-limit, and
   transport failures are distinct inconclusive categories and never masquerade as absence.
5. Registry and spec-content reads attach `GITHUB_TOKEN` as a bearer credential when present and
   allow anonymous public reads when GitHub permits.
6. Credentials and authorization headers never appear in URLs, cache keys, diagnostics, debug
   output, aggregate reports, or structured output.
7. Content bodies, per-operation work, repository concurrency, and total invocation time are
   deterministically bounded.
8. Live and cached remote content pass through the same TOML, checked-frontmatter, path, and
   identity validation before a successful verdict.
9. Existing issue verification keeps its bounded, globally deduplicated, fail-closed behavior and
   exact release-promotion invariants remain unchanged.
