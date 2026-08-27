---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: testing
---

# Testing

Four tests in `src/change_tests.rs`, beside the #711 tests they extend. Each was run with the fix
**disabled in place** — the two carry-forward assignments reverted to `None`, the monotonicity
scan short-circuited — to establish which of them are discriminators and which is not.

| Test | Unfixed | Fixed |
|------|---------|-------|
| `a_portable_definition_approval_carries_the_delta_binding_it_inherits` | FAIL: effective approval is `None` where `Some({"auth": 66d9882e..})` had just been recorded | pass |
| `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` | FAIL: no error at all — `materialize_change_deltas` returned `Ok`, `canonical_applied: true`, and `specs/auth/auth.spec.md` contained `BACKDOOR` | pass |
| `a_portable_definition_approval_records_delta_wording_with_no_prior_approval` | FAIL: the portable approve records no wording | pass |
| `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body` | **pass** | pass |

Every other test in the file passed in both states — including
`an_approval_recorded_before_delta_digests_existed_is_unknown_not_violated`,
`a_semantic_delta_swapped_after_approval_never_reaches_the_canonical_spec`, and the four
`portable_definition_*` tests that pin the pair's shape.

## Honest labels

- The **control** is `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body`,
  and it passes on the unfixed binary too. That is the point of writing it. It builds the ledger
  monotonicity is easiest to get wrong on — a pre-#711 change approved more than once, so several
  definition approvals, not one of them carrying a digest — swaps the body, and requires the
  materialization to succeed. If the refusal is ever written as "the latest approval records
  nothing" instead of "an earlier approval recorded more", this test fails, and it fails as an
  outage across every archived change rather than as a caught bug.
- `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` writes the downgraded
  ledger **directly** rather than through `change approve --portable-5-0-1`, and the test says so
  in its own doc comment. The portable projection is workflow-v1-only, and a v1 definition digest
  hashes every delta body, so on a v1 change a swapped delta is independently caught one line
  earlier by `ensure_definition_approval_valid` — unfixed, that sequence produces
  `portable definition approval pair is malformed or stale`, which is a refusal, just not this
  one. What generalizes is the shape, and under workflow v2 the shape is the whole of what stands
  between a swapped body and the canonical spec. The test asserts on the canonical spec's
  contents, not only on a message.
- `a_portable_definition_approval_carries_the_delta_binding_it_inherits` also pins the half that
  must NOT change: the pair's current and legacy digests still equal
  `portable_definition_digest_pair_v501`, the legacy note is unchanged, and
  `ensure_definition_approval_valid` still resolves the pair.

## Suites run

- `cargo clippy -- -D warnings` (bare — the form CI runs; `change check` does not run clippy)
- `cargo test` — 2395 unit tests plus 407 integration tests, all passing
- `specsync change check` and `specsync change audit --strict`

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-090 | Four tests in `src/change_tests.rs`, each run with the fix disabled in place to separate discriminator from control. `a_portable_definition_approval_carries_the_delta_binding_it_inherits` — unfixed: effective approval is `None` where `Some({"auth": 66d9882e..})` had just been recorded; it also pins that the pair's current/legacy digests still equal `portable_definition_digest_pair_v501`, that the legacy note is unchanged, and that `ensure_definition_approval_valid` still resolves. `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` — unfixed: no error at all; `materialize_change_deltas` returned `Ok` with `canonical_applied: true` and `specs/auth/auth.spec.md` contained `BACKDOOR`. `a_portable_definition_approval_records_delta_wording_with_no_prior_approval` — unfixed: the portable approve records no wording; fixed: it records `{"auth": ..}` and both `ensure_definition_approval_valid` and `ensure_approved_delta_bodies_unchanged` pass. CONTROL `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body` passes in BOTH states, which is its purpose: a pre-binding ledger holding several silent definition approvals, body swapped, materialization required to succeed. Blast radius measured rather than argued: all 197 `approvals.json` files under `.specsync/` were scanned for the refused shape and none matches. `cargo clippy -- -D warnings` bare, 2395 unit tests and 407 integration tests pass |
