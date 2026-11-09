# Recovery design checkpoint

Engineering checkpoint under SPECSYNC-FIX-AUTHORITY.json and PRODUCT-IMPLEMENTATION-TASK.md. Agent actor: Sol/Codex. Human source: Leif's reply call_b8c2de707d7c4fad940f7c1c90a28849. This is scope authority, not implementation acceptance. No product acceptance, real-site mutation, push, release or installation.

## Current-main delta inspected

Start feef2ad56afd2d6fd8f026667a8e3e701d614c63, clean. Eight commits since db641967: relayed-decision repair (#995); merge safety (#1006); upstream-archive CannotTell refusal (#967); doctor/late-close (#994); early production ownership validation (#999); runnable approval hints (#1015); disclosure that native checks do not run configured product commands (#1016); root-specs merge installation (#1017). Relevant source/guards inspected. Doctor late-close preserves an unchanged package in a separate archive directory invisible to older readers. That route is inapplicable to unmerged recovery and must not be reused to hide obsolete reopened records from 6.2.

## Strategy and order

1. Red-first frozen-reopen diagnostic and application qualification. Retain ordinary ADDED/MODIFIED base guards. Investigate a narrowly authenticated already-applied path only for audited reopened records with unchanged definition, complete anchored prior application, and a legitimate accepted later canonical successor. If these conditions cannot be proven with existing evidence, retain refusal and name retirement/replacement, never suggest editing the frozen delta. No unconditional canonical_applied bypass.
2. Add an explicit terminal nonacceptance retirement for obsolete audited-reopened records. Proposed CLI: change retire ID --actor AGENT --reason REASON --replacement ID --authority-source SOURCE. State/schema must distinguish this from Accepted/Archived success. Retain every original artifact, approval, review, failure and reopening event. A retirement event binds the original package identity and immutable definition, original/reopened historical anchor, reason, claimed authority, replacement identity and current closing evidence. Require an actual accepted/archived authenticated current replacement; refusal is atomic and repeat is idempotent only for exactly the same disposition.
3. Retirement supplies zero acceptance credit. Dependencies continue to require Accepted/Archived; successor coverage cannot use retirement as a passing historical transition. Formerly accepted obligations stay explicit and authenticated. Before enabling terminal closure, prove how they remain auditable and which legitimate successors cover changed inputs. If an anchor or obligation cannot be authenticated, stop with the exact blocker. Do not ship a disposition that simply skips that work in archive preflight.
4. New current-intent parent/replacement uses actual current contracts; the child can use existing send-back and supported dependency editing with fresh scope approval if it is an ordinary unaccepted record. Frozen children need a fresh replacement, not dependency digest mutation. Avoid adding a generic dependency-substitution ledger unless tests prove necessary.
5. ADMIN remains a genuine accepted historical-reconciliation successor. Its catalog obligation needs an exact legitimate covering successor under algorand-hub. Keep recursive authentication, current freshness, ownership, exact-base transition and unproven-history propagation. Parent-first integration is useful only when it supplies that valid edge. #621 must then recheck/review on the integrated inputs before retrying finalization.

Retirement-plus-replacement is a hypothesis, not a passing route. Implement only the authenticated case; surface any structural gap without weakening existing archive integrity. Root's parent-current fixture is read-only; no unqualified reader is run mutably against it.

## Evidence representation and compatibility

Prefer an append-only retirement record plus explicit retired terminal state in the package, preserving original raw state separately. This is a new reader capability. Keep the package enumerated by lifecycle readers; do not move it into a directory old 6.2 ignores. Old 6.2 must fail closed on the new state rather than silently certify retirement as acceptance. All consumers interpreting retirement need the qualified updated reader explicitly pinned. No claim of operational support by unchanged 6.2, and no global install. Existing legacy packages and digest preimages must stay byte/verification compatible. Confirm exact snapshot/record authentication design with failing tamper tests before enabling transition.

## Test plan and implementation boundary

Write and preserve meaningful failing tests first. Frozen ADDED/later MODIFIED must leave canonical bytes intact; diagnostic must name frozen scope and truthful recovery. Retirement must leave obsolete criterion failed and preserve exact original records. Refuse missing/stale/unreviewed replacement, malformed/unknown or tampered predecessor evidence, cycles, missing authority, exact-base/owner mismatches and retired-only dependencies. Repeat safely. Positive discriminator: authentic replacement plus eligible child rescope completes, legacy archives validate unchanged, and the legacy->ADMIN->catalog successor chain passes only with authentic coverage and keeps unproven classification.

No complete recovery is claimed before these positive discriminators pass. Product source-built reader and full Fledge verify/strict/pre-push/Trust require exact commit/tool receipts in named HC helpers with Rust 1.89, CARGO_BUILD_JOBS=2, RUST_TEST_THREADS=4 and isolated target outputs. Native structural checks are distinct from tests and human review.

## Ownership and gates

Only this product checkout plus PRODUCT-IMPLEMENTATION.md and product-evidence are writable. Sol is sole lifecycle writer. Root owns public PR creation/site histories/integration. Independent review is read-only on product source. Existing HI criteria are referenced without minting unconfirmed permanent human criterion IDs; approved additional scope wording and authority are retained in native artifacts. A new native change will record scope approval with agent actor and explicit human source. Stop before implementation acceptance/review/archive, push, release or installation.


## Concrete custody slice, after independent P1 review

Ordinary supersede continues to reject Verifying/Retired predecessors. The replacement is independently approved, checked, reviewed and finalized on current intent FIRST; it does not claim ordinary succession from the reopened record. Retirement is a distinct explicitly-unproven custody acknowledgement, never an ordinary semantic tuple or successful old implementation.

Proposed retirement.json v1 binds: domain specsync.retirement.v1; change_id; actor; authority_source (bounded recorded claim); reason; timestamp; replacement_id; original raw state bytes (UTF-8 string); full original package digest over portable relative path/mode/bytes; original reopening ledger/last prior manifest/definition digest; accepted-before-reopen receipt anchor (or exact preexisting mismatch anchor with review explicitly untrusted); replacement closing/manifest identity; sorted custody entries (path, frozen predecessor digest and owners, replacement signed entry digest and owners); implementation_accepted=false; historical_transition_status=unproven. No digest-preimage field is added to old scopes/reviews.

Every original signed input must have a signed replacement entry with compatible owners. Missing/unknown/tampered prior evidence or missing custody refuses. Normal recursive freshness of the independently accepted replacement is mandatory and revalidated when relying on the retirement. Retirement never becomes a successor candidate for another accepted change, never proves the old criterion, and never satisfies a dependency. Prior signed obligations remain discoverable in the retained prior manifest plus exact custody entries. They are acknowledged as explicitly unproven historical obligations under the named human retirement decision, rather than silently skipped. If this distinct disposition is insufficient for any incoming accepted claim, that claim still needs normal valid succession/reconciliation; retirement does not repair it.

Write order: lock -> validate unchanged audited reopen and historical source custody -> authenticate current replacement/coverage and reject circular dependencies -> capture exact package -> compute complete retirement receipt -> prepare state+receipt transaction -> move package to the ordinary enumerated archive/changes dated directory -> rollback state/receipt on move failure. All previous package files except the current state remain byte/mode unchanged; original state stays verbatim in the receipt. No earlier accepted archive is edited. Terminal state is retired, which old 6.2 cannot parse. Current roster/audit/finalize must validate retirement custody, not merely filter it away. Old reader incompatibility is an explicit CI-reader deployment dependency, not waived by this local candidate.

Initial production-path slice will test preparation/custody and one actual retirement->eligible child rescope->finalize before broad CLI/rendering changes. Exact event schema and replay validation will be pinned by those tests. Any missing earlier accepted anchor or exact successor proof stays a concrete blocker, not a waived check.

ADMIN/catalog is separate: current reconcile-history deliberately refuses a predecessor that is itself historical reconciliation, and assumes an active accepted receipt at its new definition base. Do not presume it already covers ADMIN. A minimal extension, if required, must admit an authenticated archived reconciliation receipt at the new base, preserve both complete receipt witnesses and unproven classifications, and recursively validate the earlier witness. Ordinary exact-base succession remains unchanged. First preserve the failing nested-chain test; only then qualify any implementation of this extension. No invented source behavior or padded fake delta.

## Current root decision after first positive slice

Retain COMPLETE custody until actual native qualification proves a gap. No per-entry historical-retention variant, manifest-generation relaxation or catalog reconciliation exception is being implemented. Reader archive covers BABY 4/4 by coordinator evidence, but not PARENT 151/151. Actual parent replacement manifest/path/owner coverage is still pending. Root reports a fresh properly ordered catalog successor is the ordinary site route. Source retirement custody currently requires exact owner lists, so reserved-exact/current-module owner differences will be reported as a hold rather than silently reassigned.

## Failure recovery revision after confirmed P2 red controls
Initial publication uses the existing durable two-target transaction for state plus receipt.
A failed package rename leaves both together, explicitly reports a pending archive move and
requires identical-argument retry. No unjournaled restore/delete rollback pair remains.
The old ee5e mixed layout (exact original Verifying state plus unchanged receipt) is also
resumable: construct only the intended Retired validation view, validate all committed
event bytes and current custody before any write, publish state through the transaction,
then finish the move. This view is private to retry, never a dependency or succession credit.
Default audit remains active-project only and reports archived custody not inspected;
explicit status, audit --archives and finalize retain fail-closed custody validation.

## Final authorized real-parent refinements

Authority remains Leif's bounded two-failure recovery approval, with root's explicit engineering
direction in this conversation and the existing reviewer's guard analysis. These are not human
implementation acceptance.

New receipts use schema/domain v2. Optional skipped source-kind/directory and custody-classification
fields preserve the v1 canonical preimage when absent; v1 receipts retain accepted-source and
equal-owner semantics and are never rewritten. v2 readers must validate both formats explicitly.

Owner classification v2 is same-signed-owners normally. A mismatch may be classified
unchanged-exact-file-current-owner ONLY when old owners are exactly [@exact:delivery], both
entries are ordinary File, path/kind/mode/payload/entry digests are identical, and new owners
are a nonempty vector of valid named modules with no reserved/mixed labels. Retain BOTH owner
vectors and both independently authenticated manifests. No canonical owner change, named-owner
remap or File-to-Missing exception. Recompute classification on every reliance/retry.

Source kind defaults to accepted-receipt. An explicit --source-kind
committed-audited-reopening-unproven requests only retained-record custody. Never auto-fall-through
from failed accepted authentication, and refuse this mode when accepted-state/mismatch-anchor
history exists. Require a full nonshallow bounded history, exact pre-existing ancestor package
at its real project-relative directory, Verifying/canonical-applied state and unchanged frozen
definition. Bind the full ledger, last opening, prior verification/manifest aggregate, closing
approval and finalization/review bytes. Require the prior verification commit ancestor. The
packet must reproduce the complete original pre-retirement path/mode/bytes snapshot. A stored
source anchor/directory is revalidated as such, not inferred from timestamps. No Accepted state
or recorded_review_mismatch_anchor is synthesized, and no stale-evidence cause is changed.

The explicit source proves recorded bytes and claims only; transition history stays unproven.
All current replacement, complete custody, immutable committed event, retry, dependency,
ordinary succession and legacy-baseline exclusions remain. Actual parent candidate replay
is root-owned and required before the one final full qualification lane.
