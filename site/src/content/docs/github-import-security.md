---
title: "GitHub Import Security and Limits"
section: "Reference"
order: 6
---

GitHub issue reads use typed in-process REST requests. Both a single issue import and
`specsync import --all-issues` require `GITHUB_TOKEN`; an authenticated `gh` CLI session is not a
fallback. The command may still resolve the repository from `--repo`, `github.repo`, or the local
Git remote, but provider authentication is always explicit.

Each GitHub REST operation is bounded to 10 seconds. Authentication, transport, timeout,
repository-access, and malformed-response failures stop the affected import instead of producing a
partial or false-success result. If an issue endpoint returns 404, SpecSync revalidates repository
access before deciding whether the issue is genuinely absent.

## Batch pagination

`--all-issues` requests 100 issues per page and follows encoded GitHub `Link` pagination until no
next page remains. The traversal is bounded to 100 pages. SpecSync fails the batch when:

- a `Link` header is malformed or ambiguous;
- a page repeats an issue number already returned;
- a next-page link is not a valid GitHub API URL for the expected next page; or
- page 100 still advertises another page.

These failures are deliberately not converted into a shorter successful import. A successful batch
therefore represents the complete issue list returned by the bounded provider traversal.

## Provider-process compatibility

GitHub reads, issue listing, and issue verification do not launch `gh` or another provider
subprocess. Existing `gh` authentication is not consulted. `gh` remains reserved for the explicit
drift-issue creation write path.

Set the token in the environment before importing:

```sh
GITHUB_TOKEN=... specsync import github 42 --repo owner/repo
GITHUB_TOKEN=... specsync import --all-issues --repo owner/repo
```
