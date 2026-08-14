---
change: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
artifact: testing
---

# Testing

Measured against a failing fixture and a passing control. Both directions are
required: a change that exited non-zero unconditionally would fix the failing
case and break every passing one, and the failing case alone cannot tell the two
apart.

```
failing project:  comment exit=1   body: ## ❌ SpecSync: Failed
                  check   exit=1   (agrees)
control passing:  comment exit=0   body: ## ✅ SpecSync: Passed
control gate:     comment --require-coverage 100 exit=0 on a fully covered tree
```

The last line is the one that distinguishes a real fix from a blanket
non-zero exit: the flag is honored, and honoring it still lets a covered tree
pass. A tree below the threshold exits `1`, matching `check`.

Full suite: `cargo fmt --check` clean, `cargo clippy -- -D warnings` clean,
unit and integration tests green.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-comment-005 | Failing fixture: `comment` exits 1 and its body renders `## ❌ SpecSync: Failed`, matching `check` exit 1 on the same tree. Passing control exits 0 with `## ✅ SpecSync: Passed`. `--require-coverage 100` over a fully covered tree still exits 0, which is what separates honoring the flag from exiting non-zero unconditionally |
