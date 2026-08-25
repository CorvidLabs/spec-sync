---
id: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
state: implementing
type: operations
base_commit: 0176c6a516e03f63ea83fb401d6f934ac2800a41
---

# Remove the release GitHub App from promotion: create the final tag with the workflow's own GITHUB_TOKEN and state who can now mint a release tag

## Intent

Remove the release GitHub App from promotion: create the final tag with the workflow's own GITHUB_TOKEN and state who can now mint a release tag

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The promote job of .github/workflows/release.yml creates the final tag with the workflow's own GITHUB_TOKEN and nothing else: the actions/create-github-app-token step is gone, and no file under .github/ references vars.SPECSYNC_RELEASE_APP_ID, secrets.SPECSYNC_RELEASE_APP_PRIVATE_KEY, or actions/create-github-app-token. Write access is scoped to that one job — the workflow-level permissions block stays contents: read / actions: read / checks: read, and promote declares permissions: contents: write on the job alone. The 'environment: release' reference is removed, because the release environment does not exist and referencing it makes GitHub auto-create an unprotected environment that reads in the UI as a deployment gate while enforcing nothing; the workflow records that removal and what would be required to make it a real gate. The protection given up is stated where a reader will see it, not buried: validate-release-candidate.py's UNENFORCED_TAG_POLICIES says that final-tag creation is unrestricted AND that the final tag is now minted by the workflow's own token, so anyone able to dispatch release.yml from the default branch can create refs/tags/vX.Y.Z, and that no deployment-environment gate stands between a dispatch and a release tag; release.yml still fails when that list is empty and still prints every entry as a ::warning:: annotation and into the step summary on every run, green ones included. Tag immutability is unchanged: both rulesets are still validated strictly with no bypass actor. docs/ci-confidence.md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/context.md and specs/github/tasks.md describe GITHUB_TOKEN-minted promotion with no App and no environment gate, and the open 'decide the fate of App-only final-tag creation' task is closed with the decision that was made. python3 .github/scripts/test-validate-release-candidate.py passes, its promote contract test asserts the GITHUB_TOKEN shape and the absence of every App and environment reference, actionlint reports no issue on release.yml, and cargo test passes.

## No-spec Rationale

Not applicable
