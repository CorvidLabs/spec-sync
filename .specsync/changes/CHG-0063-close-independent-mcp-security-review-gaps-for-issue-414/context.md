---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: context
---

# Context

CHG-0062 fixed issue #414's arbitrary-write primitive. Final review-driven hardening changed its
delivery inputs, so the user authorized an audited reopen. CHG-0062 is explicitly reopened and
verifying; fresh verification evidence and human closing acceptance are still pending. Its prior
immutable acceptance manifest therefore remains historical evidence rather than acceptance of the
current workspace. The current six MCP entries remain governed by CHG-0062's reopened verification
and are not modeled as CHG-0063 successor obligations.
Independent review identified medium gaps: absolute outside paths were canonicalized before
authorization, `specsync_issues` could follow redirected Git metadata, Windows reparse-point
evidence was absent, project inputs/responses were unbounded, generator write failures could report
success, and malformed JSON-RPC envelopes could dispatch.

The implementation now retains a `cap-std` server-root capability, snapshots reads into bounded
temporary directories, counts actual project/config bytes, excludes every case variant of `.git`,
uses conservative Git-freshness scoring, and rolls back partial generation. CHG-0064 owns the
`cap-std` capability dependency wiring and is explicitly reopened and verifying; fresh verification
evidence and human closing acceptance are still pending. CHG-0063 owns the `serde-saphyr`
checked-YAML dependency, review-driven behavior, tests, MCP/parser/validator contract amendments,
public docs, and release note. The current protected docs retain the intended read-only default,
write opt-in, confinement, and migration guidance while CHG-0062 awaits fresh verification and
acceptance. Expanded limits and failure semantics live in the new
`site/src/content/docs/mcp-security.md` reference.

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
contract, batch imports traverse strict GitHub pagination for at most 100 pages of 100 provider entries, and
malformed links, duplicate IDs, or a continuing page at the cap fail rather than importing a
truncated list. A new public import-security page documents the compatibility change.

The latest independent follow-up found four medium gaps after the prior clean review: issue-list
responses had no raw 100-entry page bound; Cargo snapshot discovery treated arbitrary TOML
metadata named `path` as filesystem authority; valid confined Windows-native Cargo paths were
rejected solely for using backslashes; and CLI/MCP issue scans silently omitted unreadable or
malformed specs. The follow-up restricts Cargo inputs to semantic target/workspace/dependency
tables, normalizes confined Windows paths while preserving drive/UNC/root/traversal rejection,
checks the provider-page bound before parsing entries, and makes incomplete spec inspection
inconclusive. Two previously red Windows tests were fixture defects rather than product
authorization failures: paths now use native joins, the junction target is proved before accepting
either legitimate snapshot-traversal or destination-publication confinement rejection, and the
accepted child-root case contains a valid one-file covered project.
Fresh Windows runtime and final repository/trust/provenance gates remain open.

A subsequent independent pass found remaining gaps in the issue-inspection boundary:
the lenient parser could silently filter malformed `implements`/`tracks` values, recursive spec
discovery could flatten walker failures into a partial/empty scan, and hostile filenames could
inject terminal controls or break Markdown/GitHub table code spans. Two adjacent strictness fixes
are included in the same follow-up: MCP read failures expose only sanitized relative, content-free
diagnostics, and GitHub issue-list `pull_request: null` rejects the page rather than being treated
as an ordinary issue. The final review also clarified that only top-level issue fields are
authoritative, nested extension mappings/sequences and block-scalar text are ignored, non-UTF-8
spec names cannot disappear, and CLI `specs_dir` plus all diagnostic paths remain confined and
project-relative.

The final adversarial pass closes the remaining semantic ambiguity. CLI spec discovery now walks
retained project/spec-directory capabilities and binds each discovered identity through read,
including regular-file and hardlink replacement. Retention is capped at 10,000 specs, 4 MiB per
spec, 64 MiB of spec snapshots, and a separate 4 MiB/64 MiB mapped-source ceiling. The maintained
`serde-saphyr` checked parser rejects duplicate keys or malformed YAML anywhere and
blank/null/wrong-shaped known fields while accepting comments and valid trailing commas.
`validate_spec_content_with_sources` preserves normal `issues --create` drift validation from
exact retained spec and mapped-source observations without reopening either path; the crate-private
supplied-content export seam disables ambient TypeScript wildcard resolution. Configured
repository syntax is checked even when the specs directory is missing or empty, code spans pad
edge backticks, and renderer boundaries escape bidi formatting plus Unicode line/paragraph
separators. Drift-creation terminal diagnostics and GitHub issue title/body text sanitize hostile
input. Every raw GitHub issue/pull-request item is validated as open with exact URL identity and
duplicate checks before PR filtering. CHG63 now includes explicit commands, exports, parser, and
validator requirements/deltas for these contracts. This material definition amendment requires
fresh human reapproval. Windows runtime, renewed independent rereview, repository lane, trust
verification, Attest provenance, and GitHub CI remain pending.

The renewed acceptance/adversarial cycle closed four final implementation seams. Real CLI startup
now opens and identity-binds the user-requested MCP root before any ambient canonicalization, then
requires the canonical reopen to match. CLI issue inspection keeps spec and mapped-source authority
on one retained project capability and caps the complete recursive inventory at 100,000 entries,
including non-spec files. Wrong-shaped legacy JSON `github.repo` values remain explicitly invalid
instead of falling back to Git auto-detection, and provider URLs require canonical decimal issue
numbers. The public validation API still returns rendered `Vec<String>` diagnostics; a private
structured channel preserves exact drift attribution for paths containing `": "`. These fixes and
their definition amendments are awaiting the final clean independent rereview.

The renewed acceptance audit then identified three medium evidence/compatibility gaps and one low
fixture omission: checked issue parsing did not accept CRLF delimiters, the exact-path regression
bypassed the rendered-vector compatibility route, late semantic refinements were present in deltas
but absent from the aggregate requirements/plan/summary, and boolean legacy JSON repositories were
not in the malformed-type fixture. The shared parser and CLI/MCP caller regressions now treat CRLF
equivalently to LF; compatibility tests exercise longest exact-path attribution and compile-time
public signatures; every late facet is synchronized across the selected artifacts; and all
non-string/non-null legacy repository shapes are covered. A clean rereview remains required.

The final corrected tree received two independent read-only PASS verdicts with zero high or medium
findings: one audited every acceptance/contract/evidence row, and the other rechecked security,
compatibility, and regression boundaries. Both distinguished the remaining lifecycle, hosted
Windows, trust/provenance, and GitHub CI work as procedural gates rather than implementation
findings.
