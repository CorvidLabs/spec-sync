---
change: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
artifact: context
---

# Context

Opening a change today can print a line count for `specs/<module>/context.md`, and only in human text. `change new --json` skips that pointer. `finalize` and `ship` already put `lesson_bundle` in JSON. Their text next action is only the merge instruction, and a test locks that wording so writing lessons cannot withhold a merge.

`next_questions` does not know about the working tree. The approval gate and the status line both call it. A constraints question has to be computed with the project root, and only while the change is still a draft, or every historical approval becomes an unfinished interview.

`ApprovedScopeV1.answers` is already the whole answer map. A `constraints` entry changes the scope digest for the change that records it. Older records omit the key, so their digest stays the same.

A selected plan is complete today once its placeholder comment is gone. Headings alone would pass. The new check has to require substantive text under Approach, Out of scope, Steps, Risks, and Constraints consulted, and the status list of incomplete artifacts has to use that same check.

`specsync rules` loads spec-shape configuration only. `principles_file` is hashed into the execution digest and never printed.

Do not shell out to a model. Do not fail `change new` because a context file cannot be read. Do not treat the generated context scaffold as a lesson.
