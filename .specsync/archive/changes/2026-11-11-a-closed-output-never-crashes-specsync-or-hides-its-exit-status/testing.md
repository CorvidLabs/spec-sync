---
change: a-closed-output-never-crashes-specsync-or-hides-its-exit-status
artifact: testing
---

# Testing

`tests/integration/closed_stdout.rs` starts SpecSync with its stdout reader already dropped:

- a passing `check` (text and `--json`) exits 0;
- a failing `check` exits 1, with stderr closed too;
- `watch` exits 0 on its own.

None prints panic text. All three fail on main (panic, exit 1 instead of 0, and watch running on).
