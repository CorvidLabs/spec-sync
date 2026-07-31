---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: tasks
---

# Tasks

- [x] Implement one-approval state and historical two-approval compatibility
- [x] Implement targeted bidirectional implementation verification
- [x] Implement transactional same-PR `change finalize` and one archive layout
- [x] Implement safe lightweight archive-only CI and parent-check inheritance
- [x] Implement one scoped agent review and status guidance
- [x] Separate stable scope approval from volatile execution/evidence binding
- [x] Repair strict coverage and add bounded lifecycle snapshots
- [x] Port selected schema, ignore, exports, hooks, and agents fixes
- [x] Update canonical specs, docs, version surfaces, and changelog
- [x] Run focused regression suites
- [x] Fix v2 review/finalization digest parity, v1 routing, and fork-safe review CI
- [x] Reject stable-scope contraction and explicit blocking/self-review evidence
- [x] Make scoped-review freshness commit-by-commit and post-move finalization retryable
- [x] Replace the self-referential migration with a truthful CHG-0068-only allowlisted adoption
- [x] Add the eight-originating-PR Rune regression and newcomer review documentation
- [x] Fail closed on missing CHG-0068 adoption anchors and add replay coverage
- [x] Bind append-only review attempts to required hosted-check provenance
- [x] Authenticate workflow-v2 archives after squash/rebase removes implementation objects
- [x] Recover journaled partial archive publication before state dispatch
- [x] Share review freshness limits across native and hosted validators
- [x] Freeze the one-time guard bootstrap and protect the complete workflow/Action surface
- [x] Bind first-reachable workflow-v1 eligibility to an immutable pre-v2 cutoff
- [x] Make merged-fork archive publication base-controlled and bound release history scans
- [x] Make `change adopt` move subsequent changes to workflow v2 without rewriting legacy policy
- [x] Fail adoption before mutation for cutoff-ineligible v1 records and publish migration outputs atomically
- [x] Reject committed workflow-v2 baseline deletion before any legacy fallback

Independent scoped review, final release validation, and sandbox verification are
first-class lifecycle evidence gates. They are tracked by `change status` and
`testing.md`, not as implementation tasks that would circularly block
`change check`.
