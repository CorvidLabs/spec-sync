---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: context
---

# Context

`specsync check` is the bi-directional spec<->code drift check — the product. It
does not currently run without the trust layer's permission.

`src/commands/check.rs` calls `crate::change::audit_project(root)` and, when that
report has errors, prints them and calls `process::exit(1)` **before validating a
single spec**. `audit_project` runs the fused verification-currency check: content
freshness *plus* git-ancestry walks and trusted-transition validation. So every
trust-layer failure — squash orphaning, ledger divergence, a stale evidence
commit — presents to the user as "the drift check is broken."

Measured asymmetry that motivated the chosen shape:

| | non-strict | `--strict` |
|---|---|---|
| spec drift (the product) | exit **0** (prints `1 specs checked: 0 passed, 1 failed`) | exit 1 |
| SDD/trust errors | exit **1** | exit 1 |

`EnforcementMode::Warn` is `#[default]` and documented "always exit 0 regardless of
errors or warnings" (`src/types.rs`), while the trust gate exits 1 unconditionally.
The trust layer is therefore *strictly harsher than the product it guards*: a repo
with a deleted source file and undocumented exports passes `check`, but a lifecycle
bookkeeping issue fails it outright.

This is step 1 of the 6.0 reduction, which delegates the whole trust layer to
CorvidLabs/attest. It is deliberately first: deleting trust internals before
severing this call would turn every `specsync check` in every repo with an active
change into exit 1 while the deletion is mid-flight.

What a session picking this up needs to know:

- The exit-code change (1 -> 0 for trust-red repos) is intended and user-visible.
  It is documented in CHANGELOG in this change.
- It deliberately leaves the product unable to fail by default on drift. That gap
  is closed by the immediately following change ("step 1.5"), which flips the
  default enforcement mode from `warn` to `strict`. The two are kept separate so
  the two exit-code changes bisect independently.
- Lifecycle *gating* is not being removed from the product — it moves to where it
  belongs: the `change` verbs and `change audit`.

## Why `src/change.rs` is in scope

Severing `comment.rs` orphans `check_project_quiet` in `src/change.rs` — `comment` was its
only caller. With it gone, `ConfiguredCommandOutput` retains a single live variant threaded
through three call sites for nothing.

CI runs `cargo clippy -- -D warnings`, so the orphan cannot be deferred to the later step
that deletes the surrounding subsystem. And delivery scope is immutable once the interview
is answered (`change answer` requires `draft`), so it cannot be widened in place — the first
attempt at this change was scoped to the two command files, discovered the orphan at compile
time, and had to be discarded and recreated. That is the same wall CHG-0101 hit, and is the
argument for issue #541 (cancellation as a first-class state) and for a supported way to
correct delivery scope.

The vestige is removed whole rather than trimmed: the function, the enum, and the parameter
threading all go together. Trimming inside a fused construct is what produced both reverted
fixes recorded in `docs/GOAL-6-fixes.md`.

## Blast radius, and why it took three attempts to scope

| Discovered at | What |
|---|---|
| design time | `check.rs`, `comment.rs` |
| **compile time** | `src/change.rs` — orphaned `check_project_quiet` and the `ConfiguredCommandOutput` vestige |
| **test time** | `tests/integration/{check,comment}.rs` — three tests asserting the removed behavior |

Delivery scope freezes when the interview is answered, which is before the change can be
compiled or tested. Each of the two later discoveries required editing a path the approved
scope forbade, and scope cannot be widened in place, so the workspace was discarded and
recreated twice. The full suite was then run *before* the third attempt specifically to
establish the complete radius rather than discover a fourth wall.

`correct-owner` exists to repair acceptance owners; there is no equivalent for the declared
path set. That gap is the subject of a follow-up issue, alongside #541.

## Deleted tests, not adapted

`sdd_failure_json_preserves_check_schema`, `comment_reports_sdd_only_failures`, and
`comment_reports_sdd_failures_when_no_specs_exist` are assertion-level descriptions of the
behavior this change removes — the JSON `sdd` key, the non-zero exit, and `invalid SDD
policy` in comment output. They are deleted with the behavior. Adapting such tests to pass
is what produced both reverted fixes recorded in `docs/GOAL-6-fixes.md`.
