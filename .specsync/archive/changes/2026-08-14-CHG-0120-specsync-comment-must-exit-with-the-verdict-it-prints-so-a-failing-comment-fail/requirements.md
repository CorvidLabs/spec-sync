---
change: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
artifact: requirements
---

# Requirements

`REQ-cmd-comment-005` — `comment` must terminate with the verdict it rendered.

- When the rendered body is `## ❌ SpecSync: Failed`, the process exits non-zero.
- When the rendered body is `## ✅ SpecSync: Passed`, the process exits zero.
- The exit code agrees with `specsync check` run over the same project with the
  same flags. Two commands reporting on one tree must not disagree about whether
  that tree is acceptable.
- `--require-coverage N` gates `comment` as it gates `check`, `score`, `report`
  and `deps`.

Out of scope: the content and formatting of the comment body, and the conditions
under which a comment is posted rather than printed. Both are unchanged.
