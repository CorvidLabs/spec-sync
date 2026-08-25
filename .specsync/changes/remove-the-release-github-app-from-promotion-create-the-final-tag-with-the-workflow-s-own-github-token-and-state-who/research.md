---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: research
---

# Research

Everything below was re-verified for this change rather than inherited from the previous one. A
final tag cannot be pushed to test, so the live repository and the workflow source are the evidence.

## 1. Does anything depend on a final-tag push event?

This is the question that decides whether `GITHUB_TOKEN` is safe here at all, because a push made
with `GITHUB_TOKEN` does not trigger other workflow runs.

| Workflow | Trigger | Reacts to a final tag? |
|----------|---------|------------------------|
| `release.yml` | `push: tags: v[0-9]+.[0-9]+.[0-9]+-rc.[0-9]+`, `workflow_dispatch` | No — RC tags only |
| `ci.yml` | `push: branches: [main]` (path-filtered), pull request | No |
| `pages.yml` | `push: branches: [main]` (path-filtered), `workflow_dispatch` | No |
| `trust.yml` | `pull_request`, `push: branches: [main]` | No |
| `rc-assets.yml` | `workflow_dispatch` only | No |

`release.yml` is the only workflow in the repository with a `tags:` trigger. No workflow uses
`workflow_run`, `repository_dispatch`, or a `release` event. `gh api repos/CorvidLabs/spec-sync/hooks`
returns `[]`, so no external webhook consumes the push either.

The GitHub Release itself is published by the `release` job **inside the same run**, downstream of
`promote` via `needs`, so it never depended on a tag-push event and does not now.

**Conclusion:** nothing in this repository observes a final-tag push. The standard objection to
`GITHUB_TOKEN` does not apply. It becomes a constraint only for future work that must react to
`vX.Y.Z`, which has to be called from inside `release.yml`; that is recorded at the job.

## 2. Can `GITHUB_TOKEN` actually create the tag?

`gh api repos/CorvidLabs/spec-sync/rulesets/21432148` (`SpecSync immutable final tags`):

```json
{"rules": [{"type": "update"}, {"type": "deletion"}], "bypass_actors": [], "enforcement": "active",
 "conditions": {"ref_name": {"include": ["refs/tags/v*.*.*"], "exclude": ["refs/tags/v*.*.*-rc.*"]}}}
```

No `creation` rule, so creation is permitted for any actor with `contents: write`; `update` and
`deletion` are denied to everyone, with no bypass actor. That is exactly the shape this change
relies on: the token can create the tag once and can never alter it afterwards.

## 3. Was the App ever provisioned?

- `actions/variables` → `{"total_count": 0}`
- `actions/secrets` → `{"total_count": 0}`
- `environments` → `github-pages` only

So `promote` has never executed successfully; `create-github-app-token` would have failed on an
empty `app-id`. Removing the step therefore cannot regress a working path — there is no working
path to regress.

## 4. What does a referenced-but-absent `environment:` do?

GitHub creates an environment the first time a workflow references one that does not exist, with no
protection rules attached. The result is a `release` entry in the repository's Environments list and
a deployment record per promotion — visible artifacts that read as a gate. Since the repository has
no `release` environment today, keeping the reference would have manufactured that appearance on the
first promotion. Hence removal rather than retention-with-a-comment.

## 5. Permission scoping

`permissions:` at job level replaces the workflow-level map rather than merging with it. `promote`
needs only `contents` (checkout read + tag write) and makes no `gh api` call, so `contents: write`
alone is sufficient and is the minimum. After this change the workflow grants `contents: write` in
exactly two jobs — `promote` (tag) and `release` (publication) — and the workflow-level default
stays read-only.

## 6. Tooling checks available locally

- `actionlint` 
- `shellcheck` (invoked by actionlint for `run:` blocks)
- `python3 .github/scripts/test-validate-release-candidate.py`
- `cargo test`

All four were run. Results are in `testing.md`.
