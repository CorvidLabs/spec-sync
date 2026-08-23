---
change: db-tables-without-schema-dir-must-not-gate-a-release-on-advice-the-reader-cannot-take
artifact: context
---

# Context

#684, split out of #672 after measuring that #672's fix does not address it. Different code path,
different cause, and it is the one actually blocking a real adopter.

## The trap

    src/validator.rs:2274
    if !fm.db_tables.is_empty() && config.schema_dir.is_none() {
        result.warnings.push("DB table validation skipped: `db_tables` is declared but
                              `schema_dir` is not configured")

Warnings escalate to errors under `--strict`. For a project whose schema lives in application
code — Go, in the reporting case — there is nothing to point `schema_dir` at. It is also not a
spec frontmatter field, so it cannot be scoped per module.

That leaves exactly two ways to clear it: delete the `db_tables` documentation, or abandon
`strict` and give up drift gating. **Gating a release on advice the reader cannot take is not
drift detection.**

## Why a notice rather than a suppression or a scan

Read the message: "DB table validation **skipped**". It describes what the run did NOT do. It is
not a claim about a defect in the spec, which is what warnings are for.

There was already an exact precedent — `"Planned source mapping (draft; file not created yet)"`
is carried as a **notice** for the same reason: a deliberate skip that is worth disclosing and is
not a problem.

Alternatives considered and rejected:

- **Scan the project for `.sql` files** and warn only when some exist. Plausible, but a `.sql`
  fixture or seed file is not a migration, so the signal is unreliable, and it costs a tree walk.
- **A config opt-out.** Requires the reader to know a flag exists to silence something they were
  told is their fault.
- **Suppressible via an ignore category.** Would convert a disclosure into something a project can
  silence wholesale, which is worse than the problem.

The notice keeps the disclosure, keeps the suggestion for anyone who CAN act on it, and drops a
gate that could not be satisfied.

## Measured on the adopting repository

Confirmed by a second session: `check --strict` exits 1 with exactly one warning — this one —
while plain `check` exits 0. Also narrowed: the gate is scoped to the **effective contract**, not
the repository. `fledge trust verify` passes there, and changes whose contract excludes the
declaring spec verify clean. Only changes pulling it in are stuck.

## A claim NOT carried into this change

An earlier framing said this prevents the lifecycle from ever closing on that repository. That is
**unverified**. The second session measured that 9 of 10 accepted changes advertise
`specsync change archive <id>` as a runnable next step, and nobody has executed one. The stranded
count is real; the causal story is inference. This change fixes the confirmed defect and claims
nothing about the cascade.
