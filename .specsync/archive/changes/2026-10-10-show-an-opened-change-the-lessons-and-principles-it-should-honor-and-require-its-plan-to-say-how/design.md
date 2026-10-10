---
change: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
artifact: design
---

# Design

The briefing is computed, not stored. It is not a field on `ChangeRecord` and not a field on `ApprovedScopeV1`.

```text
OpenChangeBriefing
  lessons: [{ path, lines, excerpt, truncated }]
  principles: none | { path, excerpt, truncated, missing }
```

`accumulated_lessons` stays the scaffold filter. The briefing uses that filter, then keeps a bounded prefix of the substantive lines. Twenty lines or 1,200 characters, whichever comes first, is enough to see the lesson. `truncated` is true when something was left in the file.

`principles` is present when `.specsync/sdd.json` names `principles_file`. `missing` is true when that path cannot be read. The command still succeeds.

The constraints question is last. It is absent unless the change is a draft, the other interview questions are answered, the `constraints` key is missing, and either the briefing has a lesson or a principles file is configured. Re-answering `affected_specs` deletes the key.

Plan completeness is a second predicate on top of the TODO check, applied only to `ArtifactKind::Plan`. A section body counts when it has a line that is not blank, not an HTML comment, and not a placeholder TODO. The five headings are matched case-insensitively: Approach, Out of scope, Steps, Risks, Constraints consulted.

`specsync rules` prints a "Standing principles" block after the built-in rules. It reads the SDD policy and writes nothing.
