---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: context
---

# Context

CHG-0062 fixed issue #414's arbitrary-write primitive. Final review-driven hardening changed its
delivery inputs, so the user authorized an audited reopen. CHG-0062 was reopened, freshly verified
against the implementation, and accepted again with the user's closing approval. Its historical
immutable acceptance remains preserved, and the current six MCP entries are not modeled as
CHG-0063 successor obligations.
Independent review identified medium gaps: absolute outside paths were canonicalized before
authorization, `specsync_issues` could follow redirected Git metadata, Windows reparse-point
evidence was absent, project inputs/responses were unbounded, generator write failures could report
success, and malformed JSON-RPC envelopes could dispatch.

The implementation now retains a `cap-std` server-root capability, snapshots reads into bounded
temporary directories, counts actual project/config bytes, excludes every case variant of `.git`,
uses conservative Git-freshness scoring, and rolls back partial generation. CHG-0064 owns the
`cap-std` capability dependency wiring and remains accepted/current. CHG-0063 owns the `serde-saphyr`
checked-YAML dependency, review-driven behavior, tests, MCP/parser/validator contract amendments,
public docs, and release note. The current protected docs retain the intended read-only default,
write opt-in, confinement, and migration guidance. CHG-0062 has been freshly verified and accepted
after its audited reopen. Expanded limits and failure semantics live in the new
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

Hosted Windows then exposed an 8.3-to-long-name startup-root spelling mismatch plus native fixture
and display-separator assumptions. The identity-bound original and canonical root spellings now
authorize only a lexical suffix opened through the retained canonical capability, and
sibling-prefix lookalikes remain rejected. A fresh adversarial pass also found that MCP diagnostics
reinterpreted literal Unix backslashes as hierarchy and that malformed/unreadable selected config
could fall back to defaults and claim no specs. Both are now characterized and fixed: separator
normalization is Windows-only, and checked issue config loading emits one structured non-zero
`<project-config>` finding.

The renewed review then identified an ambient double-read of CLI config, MCP allow-empty fallback
on malformed selected config, and text-only early exits under structured `issues` formats. CLI
config now uses the same retained project capability as specs, rejects link/reparse/non-regular
entries, enforces one 4 MiB identity-checked read, and parses/applies only those retained bytes with
known TOML field-type validation. MCP validates exact selected config bytes and path-selector
types before compatibility loading. Missing/empty specs and repository failures now reach the
selected JSON/Markdown/GitHub renderer. CHG-0062 and CHG-0064 are accepted/current; the prior
CHG-0063 approval and PASS verdicts predate this material artifact change, so fresh exact-digest
approval, independent rereviews, repository/trust/provenance gates, private-sandbox replay, hosted
Windows runtime, and GitHub CI remain required.

The next adversarial pass found two medium MCP config gaps: generic JSON syntax validation accepted
an array that the compatibility loader replaced with defaults, and ambient selected-config opens
could follow a link or block indefinitely on a FIFO. MCP now traverses verified regular-directory
capabilities, rejects symlink/reparse and non-regular endpoints, opens non-blocking, binds identity
before and after the bounded read, and passes the exact retained bytes through the complete checked
config parser. Focused MCP tests and installed Windows cross-target compilation pass; the final
independent rereviews and exact-tree repository/trust/provenance/hosted-CI gates remain required.

The acceptance audit additionally found that retained config parsing still invoked ambient
source-directory autodetection when the source list was omitted. Issue inspection now builds a
bounded sparse detection snapshot through the retained project capability and supplies the result
to exact-byte config parsing. A regression swaps the ambient root after the config snapshot and
proves that only the original capability's source tree is selected.

The latest acceptance and defensive reviews found six remaining implementation/evidence gaps:
selected config could be substituted between metadata inspection and open; recognized manifests
could block on special files or be silently skipped; checked JSON could accept wrong-shaped
`github.repo` through a compatibility sentinel; retained source detection diverged from the shared
ignored-name policy; direct issue details accepted pull-request payloads; and punctuation-only
issue titles produced an empty import module name. The implementation now compares opened config
and manifest identities before reading, uses non-blocking regular-file-only acquisition, validates
exact GitHub shapes before compatibility loading, shares ignore classification, surfaces special
manifests as structured findings, and rejects both direct-detail pull requests and empty import
slugs. Focused regressions pass. The prior approval digest and reviewer verdicts predate this
material amendment, so fresh exact-digest approval, two clean independent rereviews, private
sandbox evidence, final repository/trust/provenance gates, hosted Windows runtime, and GitHub CI
remain required. The private sandbox replay is now captured against implementation commit
`f6bb7a3b1aaf570b20a3a669ee2ecf46202d1f7b` and testbed commit
`758c144808d80169a44a740660b0d73c5b2f6ddd`; it passed the confined sibling drill. Fresh approval,
independent rereviews, and the remaining final gates are still required.

The next independent acceptance/defensive pass found four additional medium implementation gaps
and one evidence gap: Windows acquisition was not handle-first/no-follow, CLI config/manifests had
blocking/substitution intervals, provider-derived names could remain non-portable, partial batch
errors exited zero, and the sandbox receipt depended on an unversioned binary plus unhashed
untracked inputs. The implementation now uses retained no-follow/non-blocking handles on every
platform, validates portable names before output, returns nonzero partial-batch outcomes, and adds
the exact missing characterization tests. The earlier sandbox PASS is superseded by a successful
replay built from exact implementation commit
`b3e4696633f54ff57e42bdee7a8f20ef2bf32391`: the executable, reconstructed confined-sibling drill,
and every fixture byte are SHA-256-bound in `testing.md`; the disposable clone remained at private
testbed commit `758c144808d80169a44a740660b0d73c5b2f6ddd`, and the real private checkout remained clean.
Fresh exact-digest approval, two clean independent rereviews, audited CHG-0062/CHG-0064 evidence
refresh, final repository/trust/provenance gates, hosted Windows runtime, and GitHub CI remain
required.

A subsequent human PR review confirmed the MCP issue contract but found one medium CLI-adjacent
escape: Gradle `include` names and `projectDir` literals could normalize outside the project and
feed those source directories to coverage/check. The shared Gradle parser now rejects rooted,
drive-qualified, UNC, and parent-underflow paths before discovery. Focused parser and
multi-command structured-gate regressions pass without reading or mutating an outside fixture.
This artifact amendment supersedes the preceding approval digest and requires one fresh exact
definition approval after the concurrent independent reviews are reconciled.
