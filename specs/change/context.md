---
spec: change.spec.md
---

# Context

Canonical module maturity remains under `specsync lifecycle`; SDD delivery uses six separate states. `.specsync/sdd.json` is a dedicated versioned policy so existing projects remain opt-in. Human artifacts and deltas are Markdown, while state, approvals, and evidence are JSON. Verification commands come only from policy and run without a shell.

Numeric change allocation is additionally claimed in the committed `.specsync/change-sequence.json` ledger. The OS lock still serializes a checkout, while the ledger makes independently allocated branch claims conflict during Git integration. Lifecycle checking scans active and archived records together; the repository's immutable historical `CHG-0016` collision is acknowledged only as an exact set of full IDs.

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
Status requests renewal only when the current projection expands the approved boundary and lists
that expansion in plain language. Historical workflow-version-1 digests retain their exact
artifact-bound verification path.
