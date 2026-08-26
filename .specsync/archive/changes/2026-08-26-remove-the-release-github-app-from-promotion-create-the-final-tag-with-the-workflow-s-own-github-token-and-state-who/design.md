---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: design
---

# Design

## Shape of the change

```
                       BEFORE                                    AFTER
  workflow permissions: contents: read                  workflow permissions: contents: read
                                                                  (unchanged)
  promote:                                              promote:
    environment: { name: release }   <- absent env        (no environment)
    permissions: contents: read                           permissions: contents: write
    steps:                                                steps:
      - create-github-app-token      <- unprovisioned       - checkout (persist-credentials: false)
          app-id:    vars.…APP_ID                           - tag + push via credential helper
          private-key: secrets.…KEY                             using ${{ github.token }}
          permission-contents: write
      - checkout (persist-credentials: false)
      - tag + push via credential helper
          using steps.release-app-token.outputs.token
```

Everything below the token source is deliberately unchanged: the same `persist-credentials: false`
checkout, the same one-remote credential helper, the same idempotent
`ls-remote` → `fetch` → compare → else `tag -a` + `push` sequence, the same annotated tag message.
Only the credential changes.

## Why the credential helper stays

`persist-credentials: false` on checkout means no token is written into `.git/config`. The push then
authenticates through a `credential.helper` that exists only for the duration of the `git` process
and only for the release remote. That property is worth as much with `GITHUB_TOKEN` as it was with
an App token — arguably more, since `GITHUB_TOKEN` is present in the job environment anyway — so the
mechanism is kept verbatim and the comment above it now says why.

`x-access-token` remains the correct username for `GITHUB_TOKEN` over HTTPS.

## Why `contents: write` on the job, never the workflow

Job-level `permissions:` replaces the workflow-level map rather than merging. `promote` makes no
`gh api` call and needs no `actions:`/`checks:` scope, so `contents: write` alone is both sufficient
and minimal. Declaring it workflow-wide would hand ref-write to the other six jobs in the file — `resolve`,
`validate`, `qualify`, `record-qualification`, `authorize-release`, and `build`, several of which
check out an operator-supplied ref — for no reason.

After this change the workflow contains exactly two `contents: write` grants: `promote` and
`release`. A test pins that count so a third cannot appear unnoticed.

## Why the environment reference is deleted rather than annotated

Two audiences see two different artifacts. A workflow comment reaches the person reading
`release.yml`. An auto-created `release` environment reaches the person reading the repository's
Environments and Deployments pages, and it tells them a deployment gate exists. Since GitHub
materializes a referenced environment with **no** protection rules, retaining the reference would
have created the misleading artifact for the second audience on the first promotion, where no
comment can reach them.

Deleting the reference makes the two audiences agree: there is no gate, and neither surface claims
one. The route to a real gate is written at the job and in `specs/github/tasks.md`, in the order
that matters — environment with rules **first**, reference **second**, proving check **third**.

## Why the disclosure has three homes

The `unenforced` list is emitted per run; the workflow comment is read at audit time; the doc is
read when someone asks what the release lane guarantees. A reader of any one of the three must not
be able to conclude that a release tag implies an authority that does not exist. The list is the
enforced one — `release.yml` fails when it is empty — and the other two are prose that a test pins
by keyword (`WHO CAN MINT A RELEASE TAG`, `THE PROTECTION THAT WAS GIVEN UP`,
`NO \`environment:\` HERE, DELIBERATELY`).

Splitting the loss into two `unenforced` entries rather than extending the existing one is
deliberate: "creation is unrestricted" and "the release lane *is* the release authority" are
different facts, and a reader told only the first would not learn the second.

## Rejected alternatives

| Alternative | Why not |
|-------------|---------|
| Provision the App | The owner decided against it; that decision is the input to this change |
| Deploy key or PAT | Long-lived credential to rotate, and a workflow author reaches a repository secret exactly as easily as `GITHUB_TOKEN`. Narrows nothing |
| Keep `environment: release` with a comment | Auto-creates an unprotected environment that looks like a gate to an audience the comment never reaches |
| Workflow-wide `contents: write` | Grants ref-write to six jobs that have no reason to write a ref |
| Fold the new disclosure into the existing `unenforced` entry | Loses the distinct fact that dispatching the lane *is* the release authority |
