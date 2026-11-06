---
change: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
artifact: context
---

# Context

Issue #886, found by hosted audit S10 (2026-11-05). With `comment: true` and a read-only `GITHUB_TOKEN` (fork pull requests, or repositories set to `default_workflow_permissions: read`), the Action's Post PR Comment step calls `gh api` under `set -euo pipefail`. Creating or updating the comment returns HTTP 403, and the passing check turns red. A probe showed the lookup `GET` succeeds with `pull-requests: read`, and the create is refused even with `issues: read`.

Leif approved intent CI-3.b and this scope on 2026-11-05 at 20:30 MST, relayed by orc t18.
