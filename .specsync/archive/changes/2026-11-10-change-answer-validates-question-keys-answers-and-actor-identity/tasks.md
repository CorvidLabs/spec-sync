---
change: change-answer-validates-question-keys-answers-and-actor-identity
artifact: tasks
---

# Tasks

- [x] Reject unknown interview question keys in `answer_question_with_snapshot`, naming the advertised set
- [x] Reject empty/whitespace answers for every interview question
- [x] Trim and validate actor identity centrally in `resolve_actor` (reject empty-after-trim, embedded newlines and control characters)
- [x] Validate `affected_specs` entries against the spec registry at answer time
- [x] Update specs/change (spec, requirements, 3md lesson) and add regression tests
