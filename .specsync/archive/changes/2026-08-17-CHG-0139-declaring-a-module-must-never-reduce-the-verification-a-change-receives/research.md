---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: research
---

# Research

Four independent investigation lenses ran against this before any code changed — controlled
fixture isolation, source reading, active refutation, and blast-radius analysis. **All four
confirmed**, and the refutation lens contributed the bound that keeps the severity honest.

## What the refutation lens established

`.github/workflows/ci.yml:275` runs `cargo test --verbose` and `:382`/`:384` run both python
validators for any `src/**` or `tests/**` pull request. **Unproven code does not merge on this
repository.** The `lifecycle-gate` job is explicitly annotated as validating "without executing
verification_commands".

That backstop is repo-local, conditional on `classify.outputs.full == 'true'`, and external to the
lifecycle. Any other project adopting SpecSync has nothing equivalent, so the shipped tool is
unsound for its users even though this repository is protected.

It also ruled out the deferred-full-run explanation directly: fixtures were driven through
`check --commit` → `review` → `ship-status` → `finalize` to archived state with the configured
commands executed zero times. `accept_change_with_gate` loads the stored record and never
re-executes.

## Historical spread

Ten of 117 archived records show a narrowed set with no bare `cargo test`: CHG-0071, 0089, 0091,
0092, 0093, 0100, 0109, 0111, 0112, 0115. Two of those declared integration-test paths their
recorded filter provably could not have run. Under the current configuration, 21 of the 117 would
narrow today.

This change does not repair them. The archive is SpecSync's product, and ten of its records assert
verification that did not happen; correcting or annotating them is a separate decision.

## Detectability today

Nothing downstream can tell a narrowed run from a full one. `verification_is_current_checked`
gates on `passed` plus digests. `commands` is read in exactly one non-test place, to compose a
failure message. `change audit` emits no warning for an unrouted in-scope module, and `change show`
text output omits the command list entirely. `change show --json` does expose it, which is how the
two live changes were compared.

## The interim mitigation, and its limit

`change check --strict` restores `cargo test` and both validators, because this repository happens
to have duplicated them into `strict_verification_commands`. It does not restore `ruby --version`,
which lives only in the project list. A project that configured `verification_commands` and left
the strict list empty gets no protection from `--strict` at all.
