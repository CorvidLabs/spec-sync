---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: testing
---

# Testing

## Unit

- `strip_frontmatter_keeps_a_body_whose_horizontal_rule_is_not_frontmatter` — the truncation bug.
  A body with a rule and no frontmatter must survive whole.
- `strip_frontmatter_removes_real_frontmatter_and_keeps_later_rules` — the complementary case:
  real frontmatter goes, a later rule and everything after it stays. This test **failed on the
  first implementation** and is why the helper is delimiter-based rather than split-based.
- `accumulated_lessons_ignores_a_context_holding_only_scaffold` — a fresh scaffold must not
  advertise itself as knowledge.
- `accumulated_lessons_counts_substantive_prose_and_skips_absent_modules`.

## Dogfooded

`change new` for this very change surfaced its own modules' lessons:

```
Lessons: what these modules already learned:
  specs/change/context.md (101 line(s)) — read before scoping this change
  specs/cmd_change/context.md (21 line(s)) — read before scoping this change
```

Reading them changed the design (see `context.md`). That is the loop working end to end on
itself, not a demonstration arranged to succeed.

## Not covered

Whether an agent actually *writes* a good lesson when prompted. Out of reach of the suite; drill
032 covers `next_action` adherence generally.
