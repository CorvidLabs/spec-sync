---
change: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
artifact: context
---

# Context

`specsync comment` renders a PR comment describing whether the project passes.
It computes an exit code for exactly that purpose — the code carries a comment
saying so — uses it to select between `## ✅ SpecSync: Passed` and
`## ❌ SpecSync: Failed`, and then returns normally from the function.

Returning normally means the process exits `0`. So the command posted a comment
saying the project failed, and told its caller the project passed.

Used the way the README documents it — as a CI step that comments on a PR —
`comment` was a permanent green light. A repository whose only spec-sync step
was `specsync comment` could never fail CI, no matter how much drift accrued,
while the PR displayed a failure comment nobody's tooling acted on.

This is the third instance this session of the same meta-pattern: the intent was
written down next to the code and never delivered. #560 had a comment saying
`--strict` must gate the path it did not gate; #570 warned about a config it
then used defaults for; here the exit code exists, is correct, and is discarded.

The command was also the only one ignoring `--require-coverage`. `check`,
`score`, `report` and `deps` all exit `1` when coverage is under the threshold.
`comment` accepted the flag, printed the coverage line, and exited `0`.

Ruled out: changing the comment body, changing when `comment` posts, or making
the exit code configurable. The body was always right — it was the only honest
part of the command. The exit code is the part that lied.
