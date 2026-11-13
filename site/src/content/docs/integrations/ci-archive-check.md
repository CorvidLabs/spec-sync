---
title: "CI Archive Check"
section: "Integrations"
order: 3
---

Check in CI that one archived change is intact: its package is committed, and its history still
proves what was approved, verified and reviewed. The supported check is
`specsync change status <id> --json`. Nothing else needs to be installed or cached.

The [GitHub Action's archive gate](github-action.md) answers a different question:
whether any change is still open. Use that gate to block a merge, and this check to confirm a
change that already shipped.

---

## Check one archived change

```bash
set -euo pipefail
specsync change status "$CHANGE_ID" --json --timeout 120 > status.json
jq -e '.change.state == "archived" and .summary.terminal_evidence.validity == "authenticated-history"' status.json
```

The step passes only when both fields hold:

| Field | Passing value | What it means |
|---|---|---|
| `change.state` | `archived` | The package is in the dated archive under `.specsync/archive/changes/`. |
| `summary.terminal_evidence.validity` | `authenticated-history` | The archived approval, verification and review match what Git's history recorded for them. |

Other validity values fail the step:

- `corrupt-history`: the archive no longer matches its history. `summary.next_action` names the
  restore and the status read that confirms it.
- `historical-transition-unproven`: a legacy (workflow v1) archive whose move into the archive
  cannot be proven from history. Accept it explicitly if your project still holds such archives.

`--timeout 120` stops a read that runs past 120 seconds. It exits `124`, writes nothing, and its
JSON names the phase it reached and the command that resumes it. The step then fails, rather than
hanging the job.

In a GitHub Actions workflow, check out the history the read walks:

```yaml
- uses: actions/checkout@v5
  with:
    fetch-depth: 0
- name: Check the archived change
  env:
    CHANGE_ID: my-change-id
  run: |
    specsync change status "$CHANGE_ID" --json --timeout 120 > status.json
    jq -e '.change.state == "archived" and .summary.terminal_evidence.validity == "authenticated-history"' status.json
```

## What the read re-derives, and what it trusts

The read decides from the repository's history, not from what the archived package says about
itself.

Re-derived from Git:

- each archive's workflow-version history, from one walk of the lifecycle state paths from `HEAD`;
- the archive-introduction index: which commit added each archived package;
- whether a legacy change existed at the trusted pre-v2 cutoff, read from that commit's own tree.

Trusted:

- the repository's object graph, as `HEAD`, `refs/replace/` and `info/grafts` describe it;
- actor names in approvals and reviews. These are recorded claims, not authenticated identities.

Remembered, within limits:

- a verdict from an earlier read in the same clone. It lives in `.git/specsync/`, is reused only by
  the same SpecSync build, and only while the history it rested on is unchanged. It is not used at
  all in a shallow, grafted, replace-ref or partial clone. A fresh CI checkout has none.

Ignored:

- environment variables that point Git at other replacement or graft sources, such as
  `GIT_REPLACE_REF_BASE`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE`. The
  read clears them, so they cannot change its answer.

The read walks the history behind `HEAD`, so check out with `fetch-depth: 0`.
