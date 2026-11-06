---
id: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
state: approved
type: bug_fix
base_commit: 603a1356ae88763f5560c943361316456033ff49
---

# Show the steps between change new and approve on the landing page and name the change answer command in approve refusals (#883, D11)

## Intent

Show the steps between change new and approve on the landing page and name the change answer command in approve refusals (#883, D11)

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- The landing page Quick Start change workflow, run as written from its own init and add-spec auth lines with the by-hand steps it names, approves the definition it documents (the approved scope carries the documented acceptance criterion, spec auth and path src/auth.ts) and ends archived; the Archive gate refuses until finalize and passes wherever the page says to wait for CI (DOCS-3, DOCS-1).
- Approving right after change new refuses. The missing-acceptance-criterion refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`; the unfinished-interview refusal names every remaining question and the `specsync change answer <id> <question> <yes|no>` command for the first. Running each named command moves approval on until only the selected artifacts remain (DOCS-3.a, REQ-change-133).
- The approval guard is unchanged: every definition approve refused before is still refused; only the refusal text gains the command.
- site/src/content/docs/index.md classifies as a full-lane path and still a site path, so editing the landing page runs these tests in CI; other site pages keep site-only routing (DOCS-1.a).
- Both landing-page tests fail at the #881 archive tip 603a1356, because the page goes straight from change new to approve and the refusals name no command, and pass after the change; the README test keeps passing on the shared harness.

## No-spec Rationale

Not applicable
