---
spec: util.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/util.rs` | Unit | Levenshtein distance basics and safe regex compilation for valid/invalid patterns |

## Manual Testing

- [ ] Run `cargo test util::tests`.
- [ ] Run `specsync check --strict --require-coverage 100 --force` to confirm the helper remains covered by a spec.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Equal strings | `levenshtein` returns `0` |
| Empty left or right input | `levenshtein` returns the other side's character length |
| Invalid regex syntax | `safe_regex` returns `None` |
| Valid anchored or word-boundary regex | `safe_regex` returns `Some(Regex)` |
