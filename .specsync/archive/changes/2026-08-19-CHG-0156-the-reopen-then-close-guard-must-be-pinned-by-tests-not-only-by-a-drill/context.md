# Context

`200add8f` fixed #540 — a change reopened after finalize could never be closed again — by
admitting the archive-to-active direction in `validate_scoped_review_history_transition`. It
changed `src/change.rs` by 21 lines and **touched no test file**.

Confirmed by reverting it: with the fix removed the entire Rust suite is green — reopen 21/21,
scoped_review 5/5, finalize 6/6. And the guard's own refusal was asserted by nothing:

```
$ grep -rn 'moved evidence outside finalization' src/
src/change.rs:5339:   return Err("scoped review history moved evidence outside finalization".into());
```

One hit, in the product.

So drill 049 was the fix's only protection — and an external review demonstrated that drill 049
scores **12/0/0 on a binary with the guard deleted outright** (`return Ok(())` at the top of the
function). The drill asserts `rc=0`, `state=archived`, `archives=1`; deleting the guard
satisfies all three.

## What that means

The behaviour is correct today; this is not a live defect. What was missing is any protection
against its removal. Recorded on #648 alongside the same weakness in four other gates.

Worth being precise about the lesson: a gate self-flipping shows its assertions *changed state*,
not that they were *sufficient*. #540 is the case where nothing else was present.
