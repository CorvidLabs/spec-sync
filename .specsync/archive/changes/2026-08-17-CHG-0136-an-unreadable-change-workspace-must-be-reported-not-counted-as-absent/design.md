---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: design
---

# Design

## The shape

Make the two facts unrepresentable as one value:

```rust
pub struct UnreadableChange { pub id: String, pub reason: String }

pub struct ChangeRoster {
    pub records: Vec<ChangeRecord>,
    pub unreadable: Vec<UnreadableChange>,
}

impl ChangeRoster {
    pub fn is_degraded(&self) -> bool { !self.unreadable.is_empty() }
}

pub fn list_changes(root: &Path) -> Result<ChangeRoster, String>
```

A type change rather than a second accessor. The alternative — keep `list_changes` returning a
`Vec` and add `unreadable_changes(root)` beside it — was rejected because it preserves the actual
defect: a caller can read the roster and never learn the view is partial. The whole bug is that
the absence was invisible, and splitting the truth across two functions keeps it invisible.

## Which errors are data, which are errors

`Err` is reserved for failures leaving no partial truth: the changes directory itself unreadable,
and the two enumeration failures where the entry cannot even be named. Everything scoped to one
workspace becomes an `unreadable` entry, so the caller can report it *and* still show every
healthy change.

## Preserving the internal contract

Eleven internal callers inside `change.rs` compute digests, ledgers and successor sets, and were
written against "one bad workspace aborts the whole read". A silently short roster is worse there
than a hard error, so `list_changes_checked` is retained as a thin adapter that fails closed on
the first unreadable entry. Only the presentation layer sees the roster.

That keeps the blast radius to the seven command-layer callers rather than all eighteen.

## Exit status

A degraded roster exits non-zero. `change audit` already exits 1 on the very same tree, and two
commands disagreeing about whether one repository is healthy is the finding-identity-parity
defect (#576) this release has been closing. Rows print first, so the operator sees every healthy
change *and* the failure.

## JSON

The historical bare array is kept whenever every workspace is readable — that is every project
not already being lied to, so no consumer breaks. A degraded roster becomes an object carrying
`changes`, `unreadable` and `error`, because an array cannot say "and there were three I could
not read", and emitting a short array would be the same lie in a machine-readable channel.

**Trap found during implementation:** `cmd_change`'s tail handler prints its own `{"error": …}`
document in JSON mode. An arm that prints JSON *and* returns `Err` emits two concatenated
documents, making stdout unparseable — this very bug reintroduced one layer up. Both JSON arms
therefore print one complete document and `process::exit(1)` directly, matching the pattern the
file already uses elsewhere.

## Per-caller disposition

| caller | disposition |
|---|---|
| `list`, `status` | print roster, name unreadable, exit non-zero |
| `ship-status` | same roster, same exit; JSON gains `unreadable` |
| `ship` (no explicit id) | refuse to infer a target while any workspace is unreadable |
| lifecycle commit resolution | same — it writes a commit, so it must not guess |
| `sibling_active_change_ids` | count unreadable workspaces as siblings (fail closed) |
| verified-id lookup after verify | `.ok()?` — best-effort label, no safety consequence |
| `policy_at_comparison_base` | `?` — fail closed with the other roster readers |
