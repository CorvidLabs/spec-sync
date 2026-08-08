---
change: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
artifact: context
---

# Context

Sandbox drill `026-text-status-corrections.sh` reproduces a split integrity result on
SpecSync 6.0.0: a malformed `corrections.json` makes JSON `change show` fail closed,
while text `change status` and `change show` exit successfully and can print a
correction count.

The text paths intentionally avoid emitting correction-ledger data or digests because
those values are sensitive to cleartext-logging analysis. The repair must preserve that
boundary: it may validate ledger health, but its text diagnostic must be fixed,
non-sensitive wording and must not include ledger bytes or digest values.

Scope is limited to the correction-health projection in `src/change.rs`, the text
`show`/`status`/`list` renderers in `src/commands/change.rs`, their module contracts,
and regression coverage. JSON behavior is already fail-closed and must remain unchanged.

On 2026-08-08 the updated sandbox #17 drill passed against the local 6.0.0 WIP binary. The
sandbox board records the issue as fixed locally pending product delivery. Scoped lifecycle
completion is currently blocked by a pre-existing archive-baseline lookup failure in
`specsync change check`; that failure is outside this correction-ledger change.
