---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: docs
---

# Docs

- Preserve the current `HEAD` CLI and AI-agent pages while publishing lexical preauthorization,
  byte limits, exact envelope/resource validation, explicit `github.repo`, and fail-closed
  generation behavior in the new MCP security reference. Preserve CHG-0062's freshly reverified
  exact-byte evidence; do not rewrite accepted evidence or retain obsolete successor obligations.
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
  10-second per-operation bound, strict 100-by-100 pagination, and fail-closed malformed,
  duplicate, or cap-truncated page traversal.
- Record the compatibility amendment that confined Cargo manifest paths may normalize `..` across
  sibling crates while lexical, canonical, symlink, and junction escapes remain rejected.
- Document that Windows transaction cleanup consumes the final quarantine directory capability
  before name-based removal so init, generation, and collision rollback avoid sharing violations.
