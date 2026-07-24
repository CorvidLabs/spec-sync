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

The reconciled independent reviews identified three additional Gradle authority gaps. First, a
drive-qualified raw module identity could lose its drive marker when Gradle colon notation was
converted before validation. Second, the official method form
`project(...).setProjectDir(...)` was ignored, allowing the real project directory to disappear
from coverage. Third, a lexically confined Gradle-derived directory could still traverse a Unix
symlink or Windows reparse point during source probing. The amended contract validates raw include
and project-selector identities before colon mapping, accepts only literal
`setProjectDir(file(...))` and `setProjectDir(new File(rootDir, ...))`, fails closed on dynamic or
unsupported mutations, and resolves every derived directory component no-follow through a
retained project-root capability. This documentation amendment does not establish implementation
or test completion. Fresh definition approval, focused and full final-tree reruns, two clean
independent reviews, hosted-Windows runtime, private-sandbox refresh, repository/CI, trust, and
Attest provenance remain pending.

The exact-commit acceptance rereview then found a fourth false-green omission: Gradle
double-quoted strings containing `$name` or `${expression}` were treated as literal paths even
though Gradle resolves them dynamically. The amended parser rejects unescaped interpolation,
rejects dollars reconstructed through Unicode/octal escapes, preserves explicit escaped dollars
and Groovy single-quoted literals, and decodes supported path escapes before confinement.
Adversarial follow-up also moved Gradle build/settings selection to bounded regular non-link reads
through the retained root capability. Focused local parser and CLI/MCP regressions pass; a clean
exact-tree rereview and every final lifecycle/platform/trust gate remain pending.

The final read-only rereview of `725a50b` confirmed interpolation and retained-manifest acquisition
but found one medium constructor-token ambiguity: the parser stripped `new` and then `File`
without requiring whitespace, so dynamic `newFile(rootDir, ...)` could impersonate the official
`new File(rootDir, ...)` constructor and produce a false-green effective directory. The definition
now requires a real token boundary and focused parser plus CLI/MCP assignment/setter regressions.
Implementation remains pending fresh exact-digest definition approval.

While approval was pending, a concurrent agent patch attempted to recognize indirect/conditional
directives and ignore directives embedded in multiline strings or nested comments. Review found
that the same scanner could erase an unsupported triple-quoted include argument into `include()`
and still accepted line-leading directives nested under a multiline conditional block. Those
changes remain uncommitted and are not completion evidence. The final definition therefore binds
directive context explicitly: inert quoted/comment content is ignored, only supported top-level
statements are interpreted, and every indirect or unsupported directive form fails closed.

The next adversarial pass found two high authority gaps and three medium parser/preflight gaps.
MCP now preloads every Gradle build/settings variant exactly once through its retained no-follow,
non-blocking reader at the shared 4 MiB limit before parsing or probing; tools and resources reject
special, linked/reparse-backed, replaced, oversized, or invalid inputs. Checked CLI coverage now
opens source roots and reads bytes through retained capabilities, binds identities across
traversal, and derives file, LOC, directory-module, and flat-module results without ambient
reopening. Deterministic Unix and hosted-Windows barriers replace an observed Gradle module with a
symlink/junction before traversal and require every coverage gate to fail inconclusive without
outside bytes or partial generation. Parser closure also rejects indirect/conditional/compound
mutations, `newFile` token confusion, and unrooted drive-relative identities while masking
triple-quoted documentation and nested comments.

The next independent reviews found four additional medium gaps: lower-precedence Gradle filenames
could evade preflight; retained Gradle reads did not prove native path/opened identity continuity
at every checkpoint; directive scanning rejected unrelated valid control flow; and CLI coverage
mixed ambient root authorities while using unbounded recursive enumeration. A separate generic
MCP snapshot race could also block on or consume replacement project entries. The post-review
patch preflights every present Gradle filename, binds before/opened/after native identity, rejects
invoked unsupported inclusion APIs while preserving unrelated control flow, and uses one retained
project capability plus deterministic byte/entry/depth/UTF-8 budgets for CLI coverage. Generic MCP
project files now use no-follow, non-blocking, identity-continuous retained reads for tools and
resources. The approval ledger retains the user's earlier `2f9537c...` approval as immutable
history, but these material amendments make it stale; a new exact-tree digest and two fresh
independent rereviews are required.

The independent read-only acceptance review of exact commit `5070c954` rejected the tree because
checked coverage still read non-Gradle manifests and caller-selected spec ownership frontmatter
through ambient paths. Before/after root identity checks could not prevent a swap-read-restore
interval, and the root-swap barrier ran only after those reads. The remediation routes every
recognized manifest ecosystem, nested workspace probe, and selected coverage spec through the
retained project capability, moves the barrier immediately after root retention, and keeps the
ambient pathname only as a final replacement diagnostic. The same review found that Unix socket
fixtures unconditionally failed in restricted sandboxes; FIFO checks remain mandatory, while only
a host-denied socket fixture is skipped. Fresh exact-tree approval, two clean reviews, sandbox,
Windows runtime, repository/CI, trust, and provenance evidence remain required.

The exact-head review of `59ea788` found that selected-spec inventory and zero-config source
detection still needed stronger retained authority, Cargo/Node workspace declarations could replay
completed subtrees without a work bound, the moved early race barrier no longer exercised the
post-discovery traversal window, and hosted-Windows junction/path assertions still needed runtime
acceptance. The remediation now implements lazy autodetection for omitted `source_dirs`, retained
nested config/manifest-directory reachability, selected-spec inventory identity continuity, one
shared checked-coverage spec/source byte-and-entry budget, and bounded/deduplicated Cargo/Node
workspace expansion with completed-node memoization in manifest and MCP-specific traversal.
Coverage-internal early and post-discovery checkpoints remain distinct and gate callers propagate
their errors.

The latest review disposition is scoped rather than silently overclaimed. Two independent reviews
of exact commit `237e548` rejected it with three Medium findings: Node workspace child manifests
could mix generations during swap/read/restore, MCP accepted object-form workspaces without
`packages` and did not parse nested package manifests, and MCP/coverage traversal retained every
sibling directory handle. The amended tree consumes Node child manifests/probes through
identity-matching enumerated capabilities, strictly validates root and nested Node manifests, and
records sibling identities before sequential capability opens. Focused results pass 51 manifest,
44 validator, 117 MCP unit, and 67 MCP integration tests; the full suite passes 1,951 unit and 312
integration tests. The command-wide immutable CLI analysis snapshot and generic structured
discovery outcomes identified by the review are not implemented here; they are outside GitHub
#414's MCP boundary and remain assigned to later CLI/outcome/generation work. The prior exact
`237e548` sandbox receipt remains historical rather than final-tree evidence. Fresh post-fix
independent reviews of exact `971c89a` rejected it because completed Node workspace bases and
configured coverage roots retained handles proportional to root breadth. The amended tree releases
each verified Node base and identity-selects configured coverage roots before reopening/traversing
them sequentially; 90-base and 90-root regressions pass beneath a 64-descriptor limit. The
hash-bound exact `971c89a` binary replay against clean private sandbox commit `758c144` is now
historical. The amended full suite passes 1,953 unit and 312 integration tests. Fresh independent
rereview has one exact-`bead6d2` PASS with zero High/Medium findings; the second review remains
pending. A hash-bound exact-`bead6d2` offline build and clean private-sandbox replay passes with five
read-only default tools and 100% fixture coverage. Hosted-Windows runtime, repository/CI, completed
trust, and provenance evidence remain pending.

The second independent review of exact commit `bead6d2` rejected that candidate with one Medium:
the lexical read-root selector allowed `.git` itself (including nested and case-varied spellings)
to become the operation root, bypassing snapshot-level Git metadata exclusion. The amended
authorization rejects every ASCII-case `.git` component before opening the operation-root
capability. A unit characterization covers relative, nested, absolute, and case-varied selectors;
a real MCP integration places valid configuration and source bytes beneath each selector and
requires a content-free tool error. The prior exact sandbox receipt and first PASS are historical;
the amended tree passes 1,954 unit and 313 integration tests, release and Windows GNU cross-target
compilation, strict 100% file/LOC coverage, all 62 scores at 100/A, documentation
tests/lint/build, and editor-extension compile/package. The RustSec scan passed against the cached
1,169-advisory database after the networked refresh was unavailable. Two fresh exact-tree reviews,
the hash-bound sandbox replay, hosted-Windows runtime, trust/provenance, and GitHub CI remain
pending.
