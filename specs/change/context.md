---
spec: change.spec.md
---

# Context

Issue #751: explicit legacy reopen now evaluates the same historical manifest reconstruction archive requires, after authenticating the prior evidence. Matching current raw inputs plus an anchored commit no longer strand an unreconstructible acceptance. The append-only audit records `legacy_acceptance_unreconstructible`; sequence-history validation recognizes that cause only for manifest-less workflow-v1 evidence. Reverification creates a modern manifest. Existing ledgers retain their serialization; older binaries cannot read the new cause once a recovery writes it. Ordinary status and archive authentication are unchanged.

Canonical module maturity remains under `specsync lifecycle`; SDD delivery uses six separate states. `.specsync/sdd.json` is a dedicated versioned policy so existing projects remain opt-in. Human artifacts and deltas are Markdown, while state, approvals, and evidence are JSON. `change check` compares specs to code in-process and does not run the project's tests.

The committed `.specsync/change-sequence.json` ledger records the last numeric allocation ever made. Nothing ALLOCATES into it any more — identity is minted from the description as a slug — and it is retained so the marks it already carries cannot be lost: the gates read it and refuse a ledger that has fallen below what disk or the branch's own history already recorded. It is not read-only, and that is the part worth carrying: `floor_sequence_ledger_to_committed` still WRITES the file, from inside `git_commit_all`, raising a working-tree ledger that has fallen behind back to the committed high-water mark before staging. Every lifecycle commit runs it, so treating the ledger as immutable is how a change that edits it goes uncovered. The OS lock still serializes a checkout. Lifecycle checking scans active and archived records together; the repository's immutable historical sequence collisions are acknowledged only as exact sets of full IDs.

Historical acceptance reconstruction treats the committed sequence ledger as evidence, not a template. When immutable collision members signed one canonical collision-owner ledger, a bounded invocation-cached history lookup reuses those exact bytes after later claims advance the current ledger. The historical candidate must explicitly name the record in its same-sequence collision; ordinary records, unavailable history, and collision acknowledgements added after acceptance keep successor-aware synthetic reconstruction.

The public lifecycle remains one module for 5.0 to avoid a late high-risk refactor. Its intended internal seams are state/transitions, approvals/evidence, semantic deltas, Git/path coverage, effective-contract validation, and adoption/import. Extract those seams after 5.0 without changing the public API. Release evidence is recorded in accepted/archived change workspaces and the PR matrix rather than frozen as a permanent claim here.

`check_project` and `audit_project` share one private implementation and differ only in whether archived terminal evidence is revalidated. `change check` is spec↔code sync for one change; `change audit` is active-workspace health; `specsync check` is the product drift gate and does not walk SDD. The earlier `check_project_quiet` no longer exists: #543 severed `comment` from the trust layer.

Accepted review fixes use `change reopen`, which transitions only stale governed delivery evidence to `verifying`. The approval ledger appends a versioned reopen event containing the untouched prior verification and superseded closing approval. `canonical_applied` distinguishes re-verification from initial delivery so fresh acceptance cannot apply the semantic delta twice; it is lifecycle-only state and is excluded from definition approval digests.

For schema-v1 compatibility, false `canonical_applied` values are omitted from new persisted JSON. Definition-evidence validation recognizes both the original omitted encoding and the transitional explicit-false encoding, preserving approvals and verification created on either side of the field's introduction; true values remain durable for reopened and accepted workspaces. When explicit acceptance encounters a compatible transitional definition approval, it appends a stable approval with the same resolved human actor before the closing approval. The original evidence remains in the append-only ledger while older contract checkers see the stable digest as current.

`change check` records in-process spec↔code evidence; configured `verification_commands` are not spawned. Direct re-entry into SpecSync through a process context marker still fails once. Each run appends an immutable attempt to `verification-attempts.json`, while `verification.json` remains the latest projection so a corrected retry can succeed without erasing prior failure evidence. A later canonical change governs stale predecessor evidence only when its definition, state, semantic type, complete spec/path scope, and—once verifying—passed input-bound evidence are all current.

Semantic delta application resolves registered module paths through the committed registry before using the conventional `specs/<module>/` fallback, and rejects any unsafe registered path before preparing writes.

Effective-contract validation uses that same safe registry resolver, so verification and acceptance inspect the same canonical file that receives the delta. Canonical-successor evaluation computes the current project digest once per scan and reuses it for verifying candidates.

The sequence ledger is a protected meaningful path, so a change that edits it must cover it; no change generates a claim to cover, because nothing allocates a sequence. Historical collision acknowledgements must match the complete located ID set and every member must already be immutable in `accepted` or `archived`; mutable lifecycle states can never be acknowledged. Numeric sequences require at least four digits but have no four-digit upper bound.

Recursive Cargo verification resolves explicit `--manifest-path` selections inside the project before classifying package, `default-run`, package, or binary identity. Command tokenization handles quotes and trailing comments in pure Rust, while unsafe manifest traversal and shell syntax remain fail-closed. Registry-derived delivery scope is exact: the canonical spec and the standard requirements, tasks, context, testing, and design companions are covered without granting the containing directory. Interview parsing is question-aware so acceptance prose remains intact; multiple criteria require an explicit JSON string array, while affected specs and paths retain convenient comma/newline lists.

Accepted `public_contract` and `architecture_risk` mistakes use a separate versioned `corrections.json` ledger instead of mutating original state. The effective definition replays a validated value/digest chain, only adds deterministic artifacts, and binds correction history into later definition approvals. Correction moves the canonically applied workspace back to `verifying`; fresh definition, verification, and closing gates are required, while canonical deltas remain non-replayable. `change reopen` remains the delivery-only stale-evidence path.

Trusted correction-history scans include only remote-default references that resolve to commit objects. Git tree discovery uses literal pathspecs and NUL-delimited output so repository-relative paths containing spaces, quotes, or Unicode remain exact and cannot silently hide an accepted correction anchor.

The historical-path regression uses a quoted Unicode fixture on Unix and a Windows-valid spaced Unicode fixture on Windows, preserving the same NUL-delimited Git parsing assertion without constructing a platform-invalid filename.

Audited delivery reopen now supports one additional definition-bound repair: `change correct-owner` can append an exact path/module owner that was omitted from a historical affected-spec list. The path must already be in delivery scope and the current canonical module must explicitly own it. The correction remains in `state.json`, invalidates the definition approval, requires fresh verification and closing approval, and augments only the exact acceptance-manifest owner set without replaying canonical deltas.

Batch correct-owner extends that repair with repeated `--path`/`--spec` pairs, a JSON/TSV manifest, or `--all-missing` discovery. Every entry remains an independent sequenced `AcceptanceOwnerCorrection`; validation is per-entry and persistence is all-or-nothing so a partial batch never silently lands.

Stale accepted-change verification reasons are operator-facing diagnostics: each names the offending delivery input path and canonical owner, distinguishes uncovered inputs from inputs covered only by successors whose own evidence is stale (naming those successor IDs in sorted order), and states the concrete remediation — verify and accept the covering successor, restore a disappeared input, or run `specsync change reopen <id>`. The freshness predicates and terminal-evidence validity values are unchanged; only the human-readable reasons gained actionable content.

Accepted-transition authentication now falls back to recording anchors when no first-acceptance transition anchor matches: any commit reachable from `HEAD` or the remote default whose `state.json` records the change as accepted authenticates the transition when its verification and approvals bytes equal the current evidence and the record projects exactly onto the committed snapshot. This makes squash-merged evidence refreshes archivable — squash merges discard the original transition commits but preserve the accepted record bytes — while the evidence-key dedupe and the exactly-one-eligible rule keep missing or ambiguous evidence fail-closed.

Legacy acceptance-manifest reconstruction no longer aborts on adoption-era records whose inputs include production source with no canonical owner: an explicit `UnownedProductionSource` policy keeps current acceptance fail-closed while `reconstruct_legacy_at_anchor` assigns the exact delivery owner, so spec-less 5.0.1-era archived ledgers validate without per-repo repair. The relaxed path is structurally unreachable for changes accepted under current rules, which always carry a signed manifest.

`backfill_reopen_digests` provides the native 5.0→5.1 ledger path: deterministic, idempotent repair of 5.0.1-era reopenings (stale from the embedded prior verification, current from the superseding verification or a live manifest-aware recomputation), verified against the 5.1 schema before any write and skipped per-change when undeterminable. `load_approvals` maps the missing-field parse failure to the `specsync migrate 5.0` remediation.

Canonical module path resolution treats inert 5.0.1-era local registry stubs as absent via `load_local_registry`, so conventional `specs/<module>/` fallbacks remain available while non-inert unparsable registries keep the established fail-closed parse diagnostic.

Bounded Git candidate inspection can receive the same tracked child through a broad parent
pathspec in one batch and an exact child pathspec in another. Stage-zero observations are
accumulated as one `(mode, normalized object ID)` pair per path: exact repeats are idempotent,
while either field changing produces a deterministic conflicting-duplicate error without
replacing the first pair. Output bounds and all other index, path, and working-tree checks remain
unchanged.

Workflow-version-2 approval uses a stable scope projection rather than the mutable change package.
The projection contains intent, acceptance criteria, public-contract/risk declarations, and
affected spec/path/dependency/supersession scope. Artifacts, semantic-delta wording, tests,
canonical materialization, and lifecycle metadata bind a separate execution digest, so they
automatically stale verification and scoped review without asking the human to approve again.
Status requests renewal whenever the current stable projection adds, removes, or replaces part of
the approved boundary and lists that change in plain language. Historical workflow-version-1
digests retain their exact artifact-bound verification path.

The original CHG-0068 definition preimage was never committed, so its 6.0 stable-scope adoption
truthfully records `source_preimage_status: unavailable` and `equivalence_claim: none`. The source
approval event remains untouched. A CHG-0068-only compile-time allowlist freezes the exact
historical commit/blob, source event, adopted scope digest, authorization, and classification
digest; the independent scoped review is still mandatory before finalization. This is an explicit
audited exception, not a reusable approval migration mechanism.

Scoped review schema 2 records `pass` or `block` and MAY be recorded by the same actor as the
definition approver. SpecSync does not invent a two-person gate; GitHub remains the merge
authority for required reviewers. Distinct reviewers remain allowed. The review still checks
every descendant commit against every parent so a source change followed by a revert cannot
reuse an earlier review. Hit on CorvidLabs/corvid-bot PR 29: GitHub required zero reviews and
Trust was green, but `change review --reviewer leif` refused because `leif` had approved the
definition. That refusal was the defect. Archive recovery recognizes an accepted workspace already moved into its
dated destination and completes or restores it there, making a crash after rename retryable.

A renewed direct workflow-v2 approval supersedes and removes any one-time legacy scope adoption;
status derives its plain-language scope delta from either the direct projection or the validated
adopted projection. Archive retry discovers the unique existing dated package rather than
recomputing its location from the current date, so a post-rename retry remains valid across a
calendar rollover. Local-execution fixtures clear every hosted-CI marker recognized by lifecycle
validation, preventing ambient runner state from changing the behavior under test.

The CHG-0068 adoption is valid only with its exact allowlisted commit, base parent, and approvals
blob available; missing history fails closed. Scoped-review identity is a bounded ASCII claim, not
the trust root: every pass/block remains in `review-attempts.json`, while the official GitHub
Actions check on the exact implementation parent supplies authenticated merge provenance.
Freshness limits are loaded from `.github/scripts/lifecycle-validation-limits.json` by native and
hosted validators.

Archive terminal state and Markdown are published through the lifecycle transaction journal, which
is recovered before finalization dispatch after interruption. Workflow-v2 historical validation
normally resolves the implementation commit/tree; after squash or rebase discards that object, it
requires the exact clean archive subtree to be recorded with archived state in reachable history.

Every new lifecycle record persists an immutable workflow-origin version. Current and historical
state loading checks bounded every-parent history, so omitting the current version, downgrading it,
or reverting a downgrade cannot enter the legacy command path. Invocation-scoped caching keeps that
proof from repeating for the same change. The path set includes every bounded reachable canonical
dated archive state for the exact ID, preserving identity across archive, reopen, and cross-date
rearchive moves without accepting non-canonical paths.

Workflow-v2 adoption also writes one immutable project baseline whose cutoff is the stable remote
comparison-base ancestor when available, the current pre-adoption commit otherwise, or no commit
for an unborn repository. Its unique introduction requires the cutoff to precede the first parent,
so the same baseline remains valid after squash/rebase collapses later branch commits. Every
bounded touching commit and readable parent must retain the exact introduction bytes, preventing a
rewrite from being hidden by later restoration. A
workflow-v1 record is eligible only when the same ID/version exists at that cutoff with its origin
omitted or explicitly anchored to version 1. This preserves genuine historical records while
rejecting a first-reachable change that strips both version fields before its initial commit.
Expected negative ancestry probes capture Git diagnostics internally, so unavailable historical
objects affect evidence validity without leaking raw child-process fatal text into status output.

Correction-ledger health is a change-domain invariant, not command-rendering policy. Read-only text
views map any invalid effective definition to one safe generic diagnostic, while existing-change
definition mutations reload and validate the ledger only after acquiring the lifecycle project
lock. Keeping validation and persistence in one locked transaction removes the command-layer
time-of-check/time-of-use gap without exposing correction values, ledger bytes, or digests. Each
successful mutation also returns its validated effective definition and correction history, so the
command adapter never rereads that ledger after persistence and cannot turn success into a false
nonzero result.

The richer command result also captures normal and explicit-strict `ChangeSummary` projections
before releasing the lifecycle lock. Structured mutation output therefore cannot mix a validated
effective definition with a later invalid-ledger summary. The original record-returning
`answer_question`, `add_dependency`, and `add_supersedes_obligation` entry points remain compiled
as production domain contracts; the CLI alone uses the richer crate-private snapshot variants.

Lifecycle transactions publish a versioned count/digest journal durably before any payload, then
atomically replace payloads and clear the journal last. Backup reads distinguish not-found from all
other errors; malformed canonical journals fail closed without touching targets. Archive snapshots,
terminal restoration, and active-to-archive renames use the same durable file and directory-sync
primitives.

Squash/rebase fallback requires one non-root archive introduction whose exact path is absent from
every resolvable parent and whose subtree still matches the current archive. External post-merge
metadata binds that source introduction and finalization digest to the actual merge commit/tree;
the release gate independently reconstructs the same compact event.

Lesson-bundle assembly at archival is best-effort and never undoes a completed archive: knowledge
capture must not be able to fail a lifecycle operation. SpecSync assembles material and names the
next step; it never authors a lesson, because doing so would require shelling out to a particular
agent, and the agent that just ran `finalize` is already present.

Frontmatter ends at its CLOSING delimiter, never at the next `---` in the document. `---` is a
legal Markdown horizontal rule, so `split("---").nth(2)` truncates any body containing one, and
truncated material is indistinguishable from material nobody wrote. Two call sites inside one
feature drifted apart on exactly this.

This module now defines no frontmatter reader at all: lesson counting, archived bundles, and
artifact completeness read through `parser::strip_frontmatter`, the one canonical implementation
(#696). Do not add a local stripper back. Both of the ones deleted here failed silently and in
opposite directions — one left frontmatter in the text it counted, the other deleted body content
above a horizontal rule — and neither raised an error in either direction.

`artifact_content_is_incomplete` is an APPROVAL GATE, and it is the reason the stripper's
delimiter rule is not a cosmetic question. It counts prose lines, so anything the stripper leaves
behind is prose to it: a trailing space on the OPENING delimiter meant the `change:` and
`artifact:` lines counted as content and an artifact with nothing written in it was approved
(#716). The mirror image is worse and was not reported: a trailing space on the CLOSING delimiter
made the scan run to the first horizontal rule in the body, so a written design lost its prose and
was refused as incomplete with nothing on screen to explain why. Both are fixed in `parser`, where
the rule belongs — not here, because a gate that decides for itself what frontmatter is becomes
the fifth reader.

Two residuals, stated so they are not rediscovered as surprises. An artifact opened with `----` is
still accepted as written even when it holds nothing but frontmatter, because `----` is a Markdown
thematic break and treating it as a delimiter would cut real bodies at their first rule. And
deriving the gate from the generated scaffold instead — asking "is this still what we wrote for
you?" rather than counting prose — does not close that: a file with a mangled opener no longer
equals the scaffold, so it would read as written for exactly the same reason. What the scaffold
check already does is covered by the `<!-- TODO` short-circuit, which fires before the stripper
runs at all.

Delta file BODIES are now bound to the approval that signed them (#704). Under workflow v1 the
definition digest hashed every delta payload through `definition_artifact_snapshot`; the v2 stable
scope projection deliberately hashes intent and boundary only, and nothing replaced that binding —
`validate_delta_files` checks filenames, `project_input_digest` excludes `.specsync/changes/`, and
the descendant walk that would notice passes 0 of 107 archived reviews (#694). `approve` therefore
records `approved_delta_digests` on the definition approval event: one digest per module over the
delta file's body (line endings canonicalized — see below), with the module framed in so a body
cannot move between files and keep its digest. Materialization and acceptance verify it before `prepare_delta_application` runs, and
the materialization check sits ABOVE the `canonical_applied` short-circuit so a body that drifts
after the first application is still caught while it remains that change's evidence.

An ABSENT binding is unknown, never violated. Every approval written before the field existed
carries none — 188 of the 202 archived ledgers when this was last counted — and the check returns
early on `None` rather than inventing a verdict from evidence nobody could have written. `Option` plus `skip_serializing_if` keeps the
field out of persisted JSON when absent, so no existing digest moves and older ledgers re-serialize
byte-identically; `ApprovalRecord` is a tolerant evidence struct, not a digest preimage, which is
what makes the addition safe. Only definition gates record: closing and finalization gates review
delivery evidence, and recording a wording claim on them would be a lie in the ledger.

But absence is a property of a LEDGER, not of an event, and the version above was not (#719).
`change approve --portable-5-0-1` appended two definition approvals carrying no binding, and
`effective_definition_approval` reads the LAST definition gate, so a change that had just recorded
a digest ended up with an effective approval that recorded none — and the check returned early on
it. A compatibility path that reads "written before the binding existed" was made to read "this
approver declines to say", which is the opposite claim. Two halves fix it: every definition writer
now records what it signed, the portable pair included, and the early return is qualified — absence
is trusted only when no definition approval in that ledger records a digest. Every archived change
is in exactly that position, so history takes the untouched path; all 197 ledgers were scanned
before the refusal was written, and none carries the shape it rejects.

Carrying the binding forward was chosen over refusing the portable approve because
`--portable-5-0-1` is an adopter's only route to a 5.0.1-verifiable approval, and refusing it on a
change the current binary had already approved leaves hand-editing the ledger as the alternative.
The fix costs the projection nothing: `approved_delta_digests` is an input to neither the
definition digest, the 5.0.1 projection bytes, nor the pair ID.

The binding hashes the delta body with `\r\n` folded to `\n` (#730). It did not, and the raw-bytes
version asked a question the rest of the module answers the other way: `markdown_block_matches`
compares "ignoring line-ending style", `apply_markdown_block` re-emits every body in the target
file's own style, and `parse_delta` reads through `str::lines()`, which drops the `\r` of a CRLF
pair — so a CRLF delta and an LF delta materialize byte-identical canonical specs. A branch
approved on Linux and checked out on Windows with `core.autocrlf=true` therefore hit the #711 gate
with nothing edited, and the remedy the refusal names (re-approve) re-signs bytes the operator did
not choose and diverges again on the next handoff back. This repository could not see it, because
#715's `.gitattributes` pins keep its own deltas LF on every platform; every adopter without those
pins was exposed. Recomputing across all 198 archived `approvals.json` before the change: 25
recorded module digests, none of which moves under the normalization.

Do NOT widen that normalization. `markdown_block_matches` also trims surrounding blank lines and
horizontal whitespace, and copying that half would make the binding accept edits it exists to
refuse — trailing whitespace and blank lines are wording a reviewer signed. The digest's job is
strictly narrower than the applier's: the applier decides whether an edit is already applied, the
digest decides whether an approver read these bytes. Only the line-ending axis is safe to erase,
because it is the only one Git rewrites with no author behind it. A LONE `\r` is content for the
same reason and is deliberately kept: `text`, `eol` and `core.autocrlf` only ever convert between
LF and CRLF, so no checkout can introduce one, and `str::lines()` carries it into the canonical
spec. `parser::parse_frontmatter` preserves it for exactly this reason (#715).

The sibling sweep found nothing else. `delta_body_digests` was the only digest in the codebase
framing filesystem text: every other `read_bounded_change_text` caller parses or checks the content
instead of hashing it, and `definition_artifact_snapshot` — the input to `definition_digest`,
`execution_digest` and `project_input_digest` — takes its payload from the Git BLOB for a clean
tracked path, which Git stores LF-normalized whatever the working tree looks like. `approved_scope`
hashes record fields and no file text at all. The one place the snapshot reads working-tree bytes
is a path that is dirty or untracked, and the portable 5.0.1 projection already guards the case
where those two can diverge ("use a canonical LF release checkout").

Worth knowing before scoping the next defect in this area: on workflow v1 the downgrade does not
currently reach a canonical spec, because the v1 definition digest hashes every delta payload and
`materialize_change_deltas` validates it one line earlier. Measure that before claiming a
materialization — the report's sequence refuses on unfixed `main`, just with the wrong message
("portable definition approval pair is malformed or stale"), which points the reader at re-running
`--portable-5-0-1` and laundering the swap. The consequence that generalizes is v2's, where the
scope digest hashes intent and boundary only and this binding is all there is.

There was never a repository-wide "normalize then parse" convention to diverge from. An earlier
version of this note said there was, counted from 29 occurrences of `.replace("\r\n", "\n")` in
`src/`. Measured against the right denominator: of the 39 `parse_frontmatter` call sites outside
`parser.rs`, 21 normalized and 18 did not. A count of a pattern is not evidence of a convention,
and asserting one from a grep is the specific mistake #696 was corrected for twice.

Believing it cost a shipped Windows bug: `view.rs` read a spec raw and handed it to an LF-only
regex, so a clone with `core.autocrlf=true` failed with "Cannot parse frontmatter" on every spec.
`parse_frontmatter` now normalizes internally, which fixed all 18 sites without touching one of
them; an obligation on 18 callers would have been unenforceable and silent when broken.

Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes` (#709). The pin covers this
repository's working trees only — an adopter's repository, a tarball, or an archive extracted
without Git is not covered — so readers of lifecycle evidence must still tolerate CRLF rather than
assume the pin is in force.

A partial fix disguises its own symptom. #564 taught `parse_delta` that a `###` inside an open
item is content rather than a malformed item heading — and left the `flush` call above that
classification, so every content subheading still ended the item. One section carrying
subheadings became several items under one key, and application kept the last. The visible
damage was a spec silently losing documented behaviour, which reads as a grammar limitation, so
the follow-up issue proposed rejecting such deltas — the exact change that would have
reintroduced #564. Read the parser before believing the symptom's story about itself.

Delta bodies are approval-bound evidence: `approved_delta_digests` records a digest per module at
the definition gate, and materialization refuses a delta whose bytes changed after approval. An
approval carrying no digest is UNKNOWN, not violated — every archived change accepted before #704
predates the field, which is still most of the archive.
The guard and the flush ordering above are coupled: two archived deltas contain duplicate
`MODIFIED` keys that exist only because the old ordering split one section, so shipping the
duplicate-key refusal without the reordering would make those changes un-materializable.

A digest proves the bytes did not change after signing; it cannot prove the signature covered the
right bytes. Both failures happened on one change. `approved_delta_digests` caught its own delta
being edited during a rebase conflict — the accidental case, not an attack — and refused to
materialize until it was re-approved. What the guard could not see was that the re-approval was
granted over a TRUNCATED delta: regenerating one section with a script that preserved "everything
from the next `### SPEC SECTION` onward" dropped the trailing `## ADDED` block, because there was
no next section and an absent tail read as an empty one. The sealed record stopped naming the
requirement the change adds, and `collect_requirement_ids` returned `[]`, so the requirement
evidence gate went vacuous on the very change that added the binding.

Two consequences worth carrying. Currency and completeness are different questions and need
different mechanisms: content digests answer the first, an independent reader answers the second.
And when testing for a block heading, compare whole LINES — `"## ADDED" in text` matched the
string inside an invariant's prose and reported the block as already restored.

Two lessons recorded above were WRONG and were corrected by the change that unified frontmatter
handling: `parser.rs` did NOT handle CRLF before that change — its regex was LF-only, and
`parse_frontmatter` has normalized internally ever since, so the correction is already history —
and there is no normalize-then-parse
convention (21 of 39 call sites normalize, 18 do not). Both entered because a claim was asserted
from a grep count rather than read from the call sites, and both were folded here before the
correction landed. A lesson is read at `change new`, before anything is scoped, so a wrong one is
load-bearing in a way a wrong comment is not (#714). Prefer lessons that point at evidence over
lessons that summarise it: a stale pointer is visible, a confident summary is not.

A requirement whose implementation was WHOLLY deleted is the one the drift check cannot see, and
`REQ-change-055` was live and unsuperseded for a whole release because of it (#728). Drift measures
a spec against the code that implements it, so a requirement with zero implementing symbols
produces no finding at all: zero attributable code reads as "nothing to check" rather than
"orphaned". That is this release's most-repeated defect shape — a category empty for want of input,
read as a verdict — sitting one level up, in the tool's own model. The requirement most likely to
be stale is exactly the one nothing points at any more.

Two things generalise from fixing it. First, when a mechanism is retired, the spec text describing
it does not live in one place: the ordinal retirement (#665) added `REQ-change-086` and left seven
other statements of the deleted allocation standing — a `change.spec.md` invariant, two `context.md`
paragraphs, and clauses inside `REQ-change-022`, `-026`, `-070` and `-072`, none of which the
report naming `REQ-change-055` mentioned. Sweep the whole spec for the mechanism's vocabulary
(here: allocate, mint, next-ID, high-water, `CHG-NNNN`), not for the ID the issue names. Second,
prefer deleting a dead requirement over rewriting it once you have checked what would carry its
surviving invariant. `REQ-change-055` could have been rewritten around the read-only ledger, but
`REQ-change-070` already states the commit-side floor and `REQ-change-072` the branch-own-history
gate, so the rewrite would have been a third copy — and a restatement is exactly the kind of text
that goes stale next. `## REMOVED` is a first-class delta verb and leaves a permanent tombstone, so
deletion is recorded rather than silently dropped.

`REQ-change-071` was retired in the same pass for a different reason worth separating: it was not
orphaned by deleted code but flatly REVERSED by its own successor. It required refusing a ledger
below the default branch's published mark; `REQ-change-072` requires that a branch not be refused
for trailing the default branch and that the gate consult no remote. A successor that contradicts
its predecessor does not retire it — someone has to. Check for that whenever a requirement is
added to fix a regression in another.

## Verification is spec↔code, not the project's tests

`change check` used to spawn `sdd.json` `verification_commands` (`cargo test` on this repo). That
is CI's job. The verifier is the same in-process spec↔code pass as `specsync check`. Configured
test/build commands are not executed, so there is no verification child to reap and no Cargo
build-directory lock for this path to wait on. The wait-notice helpers remain as unused derivation
code covered by unit tests; they are not on the `verify_change` path.

What is true, and all that is claimed: Cargo's line reaches the operator on inherited stderr, and
it names neither the file, nor the holder, nor a remedy. This notice is ADDITIVE to it — the path,
the holder's PID where the platform reports one, the remediation, and an emitter that is SpecSync
rather than the child, printed before the child starts rather than after it has blocked.

Three deliberate refusals, each of which would have made the notice worse than silence:

- Nothing is inferred from elapsed time. A "probably wedged" heuristic that fires on a slow but
  healthy compile is the same defect one layer up, and this release already has eight instances of
  a check reporting success it had not verified.
- A build directory this cannot derive exactly produces NO notice: a Cargo configuration file in
  scope whose `[build]` table sets `target-dir`, `target` or `build-dir`, or whose `[env]` table
  sets one of the variables the derivation reads, or that cannot be parsed; a `--config` or
  `--manifest-path` argument; two `--target` triples; a custom target JSON; a profile that is not a
  single path component; a third-party subcommand whose flags mean something else. Naming a lock
  the command will never wait on restores the ambiguity the notice exists to remove, and a stale
  `<root>/target` left over from an earlier layout makes that a live possibility: the config check
  is a real read of the files Cargo merges, not an assumption that nobody has one.
- The holder's PID is printed only where the platform actually reports lock ownership, which today
  means Linux `/proc/locks`. macOS publishes it nowhere a process can read without spawning `lsof`
  — which contract item 5 forbids verification from doing — or hand-written Darwin `libproc` FFI
  for structs `libc` does not define. There the notice suggests `lsof` and says what `lsof`
  actually answers: it lists the processes with the file OPEN, which on macOS is all it reports,
  and a `flock` holder must be among them. Recommending it as though it named the holder would be
  the same overclaim one level down.

Stale-lock remediation (offering "the holder's parent is gone, it is safe to kill") was considered
and NOT built for the same reason: it needs the holder, so it would give different advice about
identical state on the two supported platforms.

Materialisation is once-per-change, and correcting an approved delta after that does NOT reach the
canonical tree. `materialize_change_deltas` returns early on `canonical_applied`, which is right —
it is what stops a delta being applied twice. `ensure_approved_delta_bodies_unchanged` sits
deliberately above that short-circuit, but it compares the delta against the digest on the CURRENT
approval, so editing a delta and re-approving it satisfies the guard, skips the write, and leaves
the canonical spec saying what the previous wording said while `check` reports success. That is how
#721's own corrected `REQ-change-091` sat in the delta and not in `requirements.md` through a green
`change check`; the fix was to write the corrected block into the canonical file by hand, byte-for-
byte as `apply_markdown_block` would have. Nothing detects the divergence today: the pair that is
never compared is "what the approval now says" against "what the tree already got".

The short-circuit skips MORE than the delta application, which is why a narrow "re-apply when the
digest differs" fix would not be enough (#741). `bump_spec_version` and `append_changelog` have a
single caller, `prepare_delta_application`, which is also below the flag — so a change that
re-materialises after `canonical_applied` loses its spec version bump and its Change Log row as
well as its delta. #721 lost all three: the delta wording on a re-approval, and then the bump and
the row again when a rebase resolved `change.spec.md` toward upstream on the belief that
materialisation would regenerate them. A canonical spec whose contract text changed while its
version and changelog did not is precisely the drift this module exists to prevent, and neither
`check` nor `audit --strict` can see it.

The transferable rule from #721 is narrower than "verify claims", because the claim that was
verified is not the one that broke. The issue's premise was measured after review asked, and the
CORRECTION that replaced it was then asserted from a `context.md` paragraph rather than from the
code — a paragraph #543 had made false and #738 fixed the same night. A correction is exactly as
likely to be assumed as the thing it corrects, and it arrives with more confidence, because it is
already the product of having been wrong once. When correcting a claim about the code, cite the
call site; `grep -rn check_project_quiet src/` returning nothing is the whole check, and it costs
one command.

One accepted cost of the own-group child: it is no longer in the terminal's foreground group, so a
verification command that read the controlling terminal would stop on `SIGTTIN` (or, under `stty
tostop`, on `SIGTTOU` when writing) instead of prompting — and because `wait` does not pass
`WUNTRACED`, a stopped child hangs the check rather than merely pausing. Verification commands are
non-interactive by contract item 5, so this is a cost accepted, not a case handled. Ctrl-C still
reaches the child, because the handler forwards the signal it received rather than assuming the
terminal delivered it to both.

A second accepted cost, and the lesson the first `change check` of #721 taught by failing on this
change's own test: an `flock` lives on the OPEN FILE DESCRIPTION, not on the file and not on the
process. The probe therefore holds the lock for the duration of one syscall pair, and a process
that forks concurrently can extend that acquisition until the child `exec`s. The descriptor is
`O_CLOEXEC` so the window cannot outlive the `exec`, and the CLI runs verification sequentially on
one thread, so the worst case is a real Cargo waiting microseconds longer. The same property makes
"the lock has been released" untestable at a deterministic instant from inside a multithreaded,
process-spawning test binary — an assertion on it failed once under a seven-worktree host and was
deleted rather than widened.

#741 is now decided, and the shape of the decision is worth keeping. The pair that was never
compared is not recorded anywhere — it is DERIVED. `canonical_applied` still means "materialization
ran", and the question "did it run for the delta on disk now?" is answered from the canonical
artefacts: a module's delta is applied when every item it declares is already reflected
(`delta_item_is_applied`), and its version bump and Change Log row both happened when the Change
Log names the change (`changelog_records_change`), because those two are written together by the
same two lines exactly once per (change, module). Recording a `materialized_delta_digests` beside
the flag was the issue's preferred direction and was rejected on evidence: the change record is
serialized and hashed for `definition_digest` in four places, each normalizing `canonical_applied`,
`correction_count` and `updated_at` out by hand first, so a new live field moves every one of those
digests unless normalized out in all four — and it still could not have answered for the bump or
the row, neither of which a delta digest derives.

The trap that fix had to avoid is the reason the CONTROL matters more than the discriminators here:
"always re-materialize" satisfies every discriminator #741 asks for and would rewrite every
canonical spec, re-bump every version and append a second Change Log row on every `check`. And a
second trap sits under the first: `apply_markdown_block` refuses to remove a block that is not
there — correctly, on a FIRST run — so re-running materialization over an already-applied
`## REMOVED` delta turns the silent skip into a hard error. Convergence is therefore scoped to
`record.canonical_applied`: only a change that has already materialized once may read
"already reflected" as done, and a first materialization still fires every refusal it ever did.

A predicate that returns `bool` decides how honest every caller of it can be. `scoped_review_is_current`
was a nine-term conjunction, and its git half already carried distinct reasons in a
`Result<(), String>` — but both callers discarded them with `.is_ok()`, so no reason ever escaped the
module. The cost surfaced two commits away, in `ship-status`, which could only ask a yes/no question
and therefore could not tell a review that had genuinely gone stale from one whose descendant walk
could not run at all. When a check has failure modes that differ in what the reader should DO, the
return type is where that difference has to live; discarding it at the call site is a decision made
once and paid for by every future caller.

The split that mattered here is VIOLATED versus UNAVAILABLE. The scoped-review descendant walk proves
that nothing but this change's own lifecycle records moved between the review and HEAD — a real
guarantee that content digests cannot restate, because `.specsync/changes/` and `.specsync/archive/`
are excluded from the project-input digest by design. A squash destroys `review.implementation_commit`,
so the walk has no range to walk. That is the guarantee being unobtainable, not broken, and collapsing
the two into one `false` is what let readiness report a satisfied guarantee it had never evaluated.
Only one branch of that walk is a decided negative: the one where it actually inspected a commit and
found a forbidden path. Every other branch — unresolvable HEAD, unreachable anchor, enumeration
failure, the descendant and parent bounds — is the walk declining to answer.

Content is decided before history, and the ordering is load-bearing rather than cosmetic. Both halves
can fail at once; a review that is stale by content AND anchored to a rewritten commit is stale for a
reason the reader can act on today, and answering "the walk was unavailable" about it would be true
and useless.

A silent `continue` is a diagnostic that lies. The successor walk of `validate_accepted_inputs_recursive`
refused candidates for seven different reasons and recorded one of them, so the message for the other
six was "no accepted or archived successor change covers it" — false in the field case, where the only
successor was the package `finalize` was closing, and `change audit` had just rated it `exact`. #685's
rule applies inside a walk as much as at a command boundary: the moment a check knows why it refused
something is the moment that reason is cheapest to keep, and `RejectedSuccessor` keeps it. Recording a
reason also made the reproduction self-diagnosing — the first fixed thing was the message, and the
message then named the failing check.

The anchor of a transition that history does not hold yet is a label, not a commit, and any consumer
that hands it to git gets a decided-looking negative. `authenticated_accepted_transition_for` admits the
working-tree closing evidence for exactly the package whose acceptance is not in history — the one
being closed, or an archive whose commit has not been made — and labels it `working-tree-closing-evidence`
on purpose. `semantic_tuple_transition_is_valid` used that label as a commit for `merge-base` and for the
detached-worktree read of the successor entry, so the tuple "did not hold" during the only window in
which it could have covered anything. The honest tree for that window is the working tree the evidence
was signed against (`accept_change_with_gate` checked its base ancestry against HEAD), not
`verification.commit`, which `validate_verification_for_commit_binding` says is an informational key.
The label is now a named constant so the next consumer can ask which shape it holds.

The closing token has to travel with the walk, not stop at the preflight that minted it. `PendingArchiveClose`
was forwarded to the historical-integrity preflight of the archived projection and to nothing else, so the
successor walk one line below re-authenticated the same projection as a reader — which is fine for a
first archive (the fallback admits a not-in-history package to everyone) and fatal for a re-finalize
after reopen (#540), where the fallback is gated on `is_closing`. The rule is now written on the token's
doc comment: readers pass `None`; the preflight forwards what it holds; `is_closing` keeps the token
inert for every package but the one being closed.

A map that is both "what to evaluate" and "what may cover it" cannot shrink one without shrinking
the other. `terminal_evidence_results_with_records` took one map, and the active-only audit handed
it the active listing to save loading archives — so an active legacy change could never be covered
by a finalized successor, because finalizing IS archiving. The first field round proved `finalize`;
the second, with the fixed binary, showed `audit` still calling the predecessor uncovered with the
pre-fix wording, which was the tell: no successor had been refused, none had been seen. The two roles
are now two parameters, and the audit loads archived records only when it has an active terminal
record to judge, only as candidates, and authenticates only one that declares the obligation.
