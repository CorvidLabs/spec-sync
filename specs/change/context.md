---
spec: change.spec.md
---

# Context

Canonical module maturity remains under `specsync lifecycle`; SDD delivery uses six separate states. `.specsync/sdd.json` is a dedicated versioned policy so existing projects remain opt-in. Human artifacts and deltas are Markdown, while state, approvals, and evidence are JSON. Verification commands come only from policy and run without a shell.

Numeric change allocation is additionally claimed in the committed `.specsync/change-sequence.json` ledger. The OS lock still serializes a checkout, while the ledger makes independently allocated branch claims conflict during Git integration. Lifecycle checking scans active and archived records together; the repository's immutable historical `CHG-0016` collision is acknowledged only as an exact set of full IDs.

Historical acceptance reconstruction treats the committed sequence ledger as evidence, not a template. When immutable collision members signed one canonical collision-owner ledger, a bounded invocation-cached history lookup reuses those exact bytes after later claims advance the current ledger. The historical candidate must explicitly name the record in its same-sequence collision; ordinary records, unavailable history, and collision acknowledgements added after acceptance keep successor-aware synthetic reconstruction.

The public lifecycle remains one module for 5.0 to avoid a late high-risk refactor. Its intended internal seams are state/transitions, approvals/evidence, semantic deltas, Git/path coverage, effective-contract validation, and adoption/import. Extract those seams after 5.0 without changing the public API. Release evidence is recorded in accepted/archived change workspaces and the PR matrix rather than frozen as a permanent claim here.

`check_project_quiet` shares all fail-closed validation with `check_project` but discards configured child-command output so `specsync comment` can emit a single bounded markdown protocol. Explicit verification and ordinary checks retain streamed diagnostics.

Accepted review fixes use `change reopen`, which transitions only stale governed delivery evidence to `verifying`. The approval ledger appends a versioned reopen event containing the untouched prior verification and superseded closing approval. `canonical_applied` distinguishes re-verification from initial delivery so fresh acceptance cannot apply the semantic delta twice; it is lifecycle-only state and is excluded from definition approval digests.

For schema-v1 compatibility, false `canonical_applied` values are omitted from new persisted JSON. Definition-evidence validation recognizes both the original omitted encoding and the transitional explicit-false encoding, preserving approvals and verification created on either side of the field's introduction; true values remain durable for reopened and accepted workspaces. When explicit acceptance encounters a compatible transitional definition approval, it appends a stable approval with the same resolved human actor before the closing approval. The original evidence remains in the append-only ledger while older contract checkers see the stable digest as current.

Verification rejects both direct lifecycle commands and indirect child re-entry through a process context marker. Each run appends an immutable attempt to `verification-attempts.json`, while `verification.json` remains the latest projection so a corrected retry can succeed without erasing prior failure evidence. A later canonical change governs stale predecessor evidence only when its definition, state, semantic type, complete spec/path scope, and—once verifying—passed input-bound evidence are all current.

Semantic delta application resolves registered module paths through the committed registry before using the conventional `specs/<module>/` fallback, and rejects any unsafe registered path before preparing writes.

Effective-contract validation uses that same safe registry resolver, so verification and acceptance inspect the same canonical file that receives the delta. Canonical-successor evaluation computes the current project digest once per scan and reuses it for verifying candidates.

The sequence ledger is a protected meaningful path, and every newly allocated change includes its generated claim in the affected path scope. Historical collision acknowledgements must match the complete located ID set and every member must already be immutable in `accepted` or `archived`; mutable lifecycle states can never be acknowledged. Numeric sequences require at least four digits but have no four-digit upper bound.

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

Scoped review schema 2 records `pass` or `block`, rejects the scope approver as reviewer, and
checks every descendant commit against every parent so a source change followed by a revert cannot
reuse an earlier review. Archive recovery recognizes an accepted workspace already moved into its
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

Lifecycle transactions publish a versioned count/digest journal durably before any payload, then
atomically replace payloads and clear the journal last. Backup reads distinguish not-found from all
other errors; malformed canonical journals fail closed without touching targets. Archive snapshots,
terminal restoration, and active-to-archive renames use the same durable file and directory-sync
primitives.

Squash/rebase fallback requires one non-root archive introduction whose exact path is absent from
every resolvable parent and whose subtree still matches the current archive. External post-merge
metadata binds that source introduction and finalization digest to the actual merge commit/tree;
the release gate independently reconstructs the same compact event.
