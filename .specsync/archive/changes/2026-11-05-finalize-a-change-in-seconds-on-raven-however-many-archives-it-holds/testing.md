---
change: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-121 | `finalize_git_processes_do_not_grow_with_other_archives` failed first on #840 (four more archives added 654 Git processes to one finalize, 1,296 with few and 1,950 with four more) and passes with at most 12 per archive. The full change suite, including every finalize and archive refusal test, passes unchanged. |
