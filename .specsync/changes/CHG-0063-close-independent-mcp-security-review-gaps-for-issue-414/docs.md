---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: docs
---

# Docs

- Preserve the current `HEAD` CLI and AI-agent pages while publishing lexical preauthorization,
  byte limits, exact envelope/resource validation, explicit `github.repo`, and fail-closed
  generation behavior in the new MCP security reference. Record that CHG-0062 has fresh
  verification and closing acceptance after its audited reopen; preserve its historical
  exact-byte evidence without retaining obsolete CHG-0063 successor obligations.
- Add an `[Unreleased]` security/compatibility entry to `CHANGELOG.md` for the read-only default and
  review-driven hardening.
- Keep the existing `--allow-write` migration guidance and make explicit that GitHub read/list/
  verify operations require `github.repo` plus `GITHUB_TOKEN` and do not execute `gh`; `gh` remains
  only for explicit issue-creation writes.
- Document that issue verification is repository-preflighted, globally deduplicated/capped, and
  inconclusive on provider/authentication/transport failures rather than false-green not_found.
- Publish the exact 4 KiB request-ID, 1,000-spec/64 MiB generation, 100-issue, 10-second operation,
  and 30-second complete-batch limits in `site/src/content/docs/mcp-security.md`.
- Publish `site/src/content/docs/github-import-security.md` without further changing the protected
  CLI page. Document explicit-token single/batch imports, no authenticated-`gh` fallback, the
  10-second per-operation bound, strict 100 pages by 100 raw provider entries, and fail-closed
  oversized, malformed, duplicate, or cap-truncated page traversal.
- Record the compatibility amendment that confined Cargo manifest paths may normalize `..` across
  sibling crates and may use Windows-native backslashes while drive, UNC, rooted, traversal,
  canonical, symlink, and junction escapes remain rejected. Clarify that only semantic Cargo
  target/workspace/dependency path tables authorize snapshot inputs; arbitrary metadata `path`
  keys do not.
- Document that Windows transaction cleanup consumes the final quarantine directory capability
  before name-based removal so init, generation, and collision rollback avoid sharing violations.
- Record that GitHub list pages are capped at 100 raw provider entries before pull-request
  filtering or item parsing and reject a present null/non-object `pull_request` marker. Record that
  CLI/MCP issue verification checks only top-level `implements`/`tracks`, ignores nested extension
  and block-scalar lookalikes, and is inconclusive when discovery encounters a walker/non-UTF-8
  filename failure or any discovered spec is unreadable or has malformed/missing frontmatter.
- Document that CLI `specs_dir` is confined beneath the project and all inspection diagnostics are
  bounded, content-free, and project-relative.
- Document that CLI issue discovery binds discovered identity through read, including regular-file
  and hardlink replacement, and retains at most 10,000 specs, 4 MiB per spec, and 64 MiB
  cumulatively. Document the separate 4 MiB per mapped source / 64 MiB cumulative source ceiling.
- Document that `issues --create` preserves normal drift validation through
  `validate_spec_content_with_sources` and `SourceSnapshot` without reopening retained spec or
  source paths; supplied-content export extraction does not resolve TypeScript wildcard imports
  through ambient paths.
- Document the maintained `serde-saphyr` checked issue parser: duplicate/global malformed YAML and
  blank/null/wrong-shaped known fields fail closed; comments and valid trailing commas work;
  nested extension and block-scalar lookalikes are ignored; LF and CRLF frontmatter delimiters are
  accepted equivalently.
- Record that configured repository syntax is validated even with zero references or a
  missing/empty specs directory, while Git auto-detection/provider access remain skipped. Record
  that renderer sanitization covers bidi formatting plus Unicode line/paragraph separators, that
  Markdown code spans pad leading/trailing backticks, and that drift-creation terminal diagnostics
  plus GitHub issue title/body text sanitize hostile input.
- Record that every raw GitHub issue/pull-request item is fully validated as open with exact
  repository/resource/number URL identity and raw duplicate checks before PR filtering.
- Document that CLI issue config is one retained, same-handle, 4 MiB snapshot parsed from exact
  bytes; linked/non-regular/replaced/malformed/wrong-shaped config fails closed. MCP likewise
  acquires selected config through a no-follow, non-blocking, identity-verified regular-file
  snapshot and passes the exact bytes through complete checked parsing before compatibility
  loading.
- Document that omitted CLI issue source directories are detected through a bounded sparse
  retained-capability snapshot rather than a replaceable ambient root path. The snapshot applies
  the shared ignored-directory policy before metadata inspection and reports recognized
  non-regular manifests as inconclusive.
- Record that selected config and recognized manifest files must remain regular and identity-stable
  through explicit no-follow, non-blocking retained-handle reads on Windows and Unix; exact checked
  JSON validates the `github` object and `github.repo` type before compatibility substitution.
- Record that direct issue-detail reads reject pull-request payloads and imports reject issue
  titles that cannot produce a portable module name. Batch imports continue later items but exit 1
  after any error.
- Record the exact implementation/testbed revisions plus executable, drill, and fixture hashes for
  every private-sandbox replay; do not treat mutable untracked inputs as reproducible evidence.
- Document that missing/empty specs and repository-resolution failures use the selected
  JSON/Markdown/GitHub renderer instead of text-only early exits.
- Document the Gradle compatibility/security amendment: raw drive-qualified module identities are
  rejected before colon mapping; assignment-style `projectDir` and official literal
  `setProjectDir(file(...))` / `setProjectDir(new File(rootDir, ...))` share one confined parser;
  dynamic or unsupported mutations fail closed; and each derived directory component is checked
  no-follow through the retained root capability before CLI/MCP source traversal.
- Document that unescaped or Unicode/octal-encoded interpolation in double-quoted Gradle strings
  fails closed, explicit literal-dollar forms remain compatible, and present Gradle build/settings
  files are bounded regular non-link reads through the retained root capability.
- Document that every present Gradle filename is preflighted before precedence and identity-bound
  through read; unsupported invoked inclusion APIs fail closed while unrelated control flow stays
  compatible.
- Publish the retained CLI coverage limits (8 MiB/file, 64 MiB total, 100,000 entries, 256
  components) and explain that caller-selected spec mappings, every recognized manifest/workspace
  probe, spec-module enumeration, and source traversal share one project authority.
- Document that generic MCP project files use no-follow, non-blocking, identity-continuous reads
  for both tools and resources and reject special/link/replacement races without attacker bytes.
- Keep fresh Windows runtime and final repository/trust/provenance evidence described as pending
  until those gates actually pass.
