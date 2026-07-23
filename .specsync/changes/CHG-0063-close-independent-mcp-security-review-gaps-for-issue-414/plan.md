---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: plan
---

# Plan

1. Reject absolute outside roots before canonicalization, retain the server-root directory
   capability, and require explicit MCP issue-repository identity instead of consulting project Git
   metadata.
2. Validate complete JSON-RPC 2.0 envelopes and exact resource arguments before dispatch.
3. Build bounded read snapshots with actual-byte accounting, including configuration inputs, and add
   deterministic inbound-line and outbound-response bounds; propagate transport errors.
4. Preflight capability-relative generation destinations, verify every required output, and roll
   back partial multi-file writes on failure.
5. Add Unix symlink, Windows junction, case-insensitive Git metadata, malformed envelope/resource,
   byte-budget, response-ID-bound, and transactional write-failure regressions.
6. Update the MCP semantic delta, companions, public documentation, and release changelog.
7. Run focused tests, full repository/trust gates, private sandbox replay, two independent rereviews,
   Attest provenance, and GitHub CI before requesting closing approval.
8. Close follow-up adversarial findings by identity-binding startup root acquisition, preserving
   manifest-derived inputs across ignores, bounding and atomically publishing generated output, and
   adding Windows write-junction coverage; rerun every affected gate afterward.
9. Close final acceptance findings by capturing the retained root handle before canonicalization
   and retaining failed empty parents rather than claiming ownership across create/open races.
10. Close the next independent findings by parsing/budgeting every manifest and making GitHub issue
    verification repository-aware, typed, globally deduplicated, capped, strict, and time-bounded.
11. Close final adversarial findings with capability-only read-root resolution, no provider
    subprocess for GitHub reads/listing/verification, post-404 repository revalidation, immutable
    preflighted manifest bytes, one shared full Gradle settings parser, identity-bound generation
    rollback, truthful all-error summaries, and complete public compatibility limits.
12. Close the final dual-review findings with scoped quarantine-based rollback, conservative empty
    parent retention, staged-identity publication checks, real TOML Cargo workspace parsing, one
    Windows-aware root suffix routine, in-process GitHub reads, fail-closed Gradle parsing, and
    no-reference issue reporting before repository resolution.
13. Close renewed acceptance findings by accepting CRLF checked frontmatter, testing the real
    rendered-`Vec<String>` compatibility path, adding `REQ-config-006` for every malformed legacy
    JSON repository type, retaining one project capability for specs and mapped sources, enforcing
    the 100,000-total-entry bound, requiring canonical decimal provider URL spelling, and
    synchronizing all six facets across definition artifacts; rerun both independent reviews and
    all gates afterward.
14. Close hosted-Windows and final adversarial findings by accepting identity-bound startup-root
    aliases without trusting ambient candidates, preserving literal Unix diagnostic backslashes,
    normalizing separators only on Windows, and making malformed/unreadable selected configuration
    a structured non-zero issue-inspection finding; synchronize the definition and rerun every
    exact-revision gate and both independent reviews.
15. Close the renewed config/output findings by snapshotting CLI config through the same retained
    project capability, enforcing same-handle identity and a 4 MiB bound, parsing exact JSON/TOML
    bytes with known-field type checks, validating MCP selected configs before compatibility
    loading, and routing missing-spec/repository failures through the selected structured renderer.
16. Close the final adversarial selected-config and omitted-source findings with non-blocking,
    no-follow, identity-verified regular-file snapshots, complete checked parsing, and
    capability-derived source detection; rerun the exact-tree independent reviews and every final
    gate.
