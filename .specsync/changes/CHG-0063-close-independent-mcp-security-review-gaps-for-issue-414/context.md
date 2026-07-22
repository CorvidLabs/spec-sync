---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: context
---

# Context

CHG-0062 fixed issue #414's arbitrary-write primitive. Final review-driven hardening changed its
delivery inputs, so the user authorized an audited reopen. CHG-0062 was exactly reverified against
the final current workspace and reaccepted with fresh human closing approval. Its latest immutable
acceptance manifest therefore already signs the six current MCP entries that an earlier CHG-0063
draft had modeled as successor obligations. Those obligations became obsolete after reacceptance
and were removed from this still-unapproved definition rather than rewriting accepted evidence.
Independent review identified medium gaps: absolute outside paths were canonicalized before
authorization, `specsync_issues` could follow redirected Git metadata, Windows reparse-point
evidence was absent, project inputs/responses were unbounded, generator write failures could report
success, and malformed JSON-RPC envelopes could dispatch.

The implementation now retains a `cap-std` server-root capability, snapshots reads into bounded
temporary directories, counts actual project/config bytes, excludes every case variant of `.git`,
uses conservative Git-freshness scoring, and rolls back partial generation. CHG-0064 owns the direct
runtime dependency wiring and was likewise exactly reverified and reaccepted after the authorized
audit reopen. CHG-0063 owns the review-driven behavior, tests, MCP contract amendment, public docs,
and release note. The current protected docs match the freshly accepted CHG-0062 evidence and retain
the intended read-only default, write opt-in, confinement, and migration guidance. Expanded limits
and failure semantics live in the new `site/src/content/docs/mcp-security.md` reference.

A second adversarial review found a startup root-acquisition interval, ignored-directory snapshot
omissions, direct final-path generation writes, unbounded generation output, and missing Windows
write-junction coverage. CHG-0063 now also identity-binds root opening, preserves root-wide and
manifest-derived inputs, and stages/syncs bounded generation output before atomic publication.
The review's GitHub issue-fetch false-green is handled here because it is exposed by the MCP
`specsync_issues` tool: provider, authentication, and transport failures now make the tool
inconclusive instead of returning a successful empty result. Issue #419 remains separately scoped
to dependency coverage and `depends_on` validation.

Final acceptance review tightened two remaining details: the first root handle and identity are now
captured before canonicalization and compared with the reopened canonical path. Generated file
rollback preserves public replacements, while failed empty parents are conservatively retained
rather than claimed across the create/open interval.

The next independent pass found multiline Cargo workspace omission, unbudgeted manifest discovery,
ambiguous GitHub repository/issue failures, and unbounded duplicate fanout. CHG-0063 now also owns
`src/github.rs`, `src/commands/issues.rs`, and their contracts: project-wide issue verification
preflights repository access, uses typed outcomes, strictly parses responses, globally
deduplicates/caps IDs, and bounds REST requests and whole-batch time.

The final review passes exposed path replacement after read-root selection, escapable provider
subprocesses, ambiguous 404s after access revocation, mutable manifest rereads, omitted Gradle
forms, same-entry publication/rollback races, Windows absolute-root representation mismatches, and
repository resolution before confirming any references. Read roots now resolve through the
retained root handle; issue reads/listing/verification use in-process REST with explicit
`GITHUB_TOKEN` and no provider subprocess; missing issues trigger an in-budget repository recheck;
snapshots copy immutable preflight buffers; Cargo workspace selection is parsed as TOML; checked
Gradle discovery is comment/escape-aware; publication and rollback atomically quarantine current
entries before identity checks; Windows roots share normalized drive/extended/UNC matching; and the
issues command gathers references before repository resolution.

Generation now retains empty parents created by a failed batch instead of claiming ownership across
the non-atomic create/open interval. The caller/path boundary excludes same-user races against
private stage/quarantine names by a process already authorized to mutate the server root.

The last adversarial import review found that the command-level import path still inherited
provider-process authentication and could return a partial issue set. CHG-0063 therefore also owns
`src/importer.rs`, `importer`, and `cmd_import`: single imports use the shared typed issue-detail
contract, batch imports traverse strict GitHub pagination for at most 100 pages of 100 issues, and
malformed links, duplicate IDs, or a continuing page at the cap fail rather than importing a
truncated list. A new public import-security page documents the compatibility change.
