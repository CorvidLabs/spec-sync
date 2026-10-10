---
id: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
state: implementing
type: feature
base_commit: 684aed366163c29ce3b5df22b3c4716353d93fcf
---

# Show an opened change the lessons and principles it should honor, and require its plan to say how

## Intent

Show an opened change the lessons and principles it should honor, and require its plan to say how

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cmd_rules`

## Acceptance Criteria

- When I open a change, I can see the lessons already written for the modules it touches, and the project's principles when I have any.
- My agent sees the same thing in the structured result.
- A module that still has only its original scaffold is not presented as if it had learned something.
- A change that needs a plan cannot be approved until that plan says what I will do, what I will not do, the steps, the risks, and which existing constraints I am honoring.
- A change the interview did not select a plan for still does not grow one.
- While a change is still a draft, and only when lessons or principles exist, I am asked which of them constrain it, and "none" is a complete answer.
- Changing that answer is a change of the agreed scope.
- When a change is archived I am shown where its lesson bundle is, and I can still merge without copying those lessons into the spec.
- A lesson becomes a standing rule only when I write it into the project's principles, and the next change shows me that file before I approve it.

## No-spec Rationale

Not applicable
