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

`--all-issues` requests up to 100 raw provider entries per page and follows encoded GitHub `Link`
pagination until no next page remains. The traversal is bounded to 100 pages. The raw-entry bound is
checked before pull-request filtering or item parsing, so pull requests still consume the page
budget. SpecSync fails the batch when:

- a page contains more than 100 raw provider entries;
- a present `pull_request` marker is `null` or any value other than an object;
- a `Link` header is malformed or ambiguous;
- a page repeats an issue number already returned;
- a next-page link is not a valid GitHub API URL for the expected next page; or
- page 100 still advertises another page.

Before pull requests are filtered from an otherwise valid page, every raw item is validated. Its
`pull_request` marker shape, positive numeric identity, nonempty title, nonempty names for any
labels, exact `open` state, and canonical
`https://github.com/{owner}/{repository}/issues/{number}` or `/pull/{number}` URL must agree.
Duplicate raw identities fail within and across pages even when one or both entries would otherwise
be filtered as pull requests.

These failures are deliberately not converted into a shorter successful import. A successful batch
therefore represents the complete issue list returned by the bounded provider traversal.

## Related issue-verification compatibility

CLI and MCP issue verification share a maintained `serde-saphyr` real-YAML parser for top-level
`implements` and `tracks`. Duplicate keys or malformed YAML anywhere, and blank, null, scalar,
mapping, mixed, non-positive, or overflowing known fields, make inspection inconclusive. Comments
and valid trailing commas are accepted. Nested extension mappings/sequences and block-scalar text
with issue-like keys are ignored.

The CLI opens the project and configured specs directory through retained capabilities, then reads
each immutable spec snapshot through the same identity-checked file handle. `specsync issues
--create` validates those exact bytes through the shared validator without reopening discovered
spec paths, preserving normal drift-issue creation for stable snapshots. With no references, the
CLI skips Git auto-detection and provider access but still validates the syntax of an explicitly
configured repository. Human, JSON, Markdown, and GitHub renderers escape terminal controls,
bidirectional formatting controls, and Unicode line/paragraph separators; Markdown/GitHub output
also preserves valid escaped table cells and code spans.

## Provider-process compatibility

GitHub reads, issue listing, and issue verification do not launch `gh` or another provider
subprocess. Existing `gh` authentication is not consulted. `gh` remains reserved for the explicit
drift-issue creation write path.

Set the token in the environment before importing:

```sh
GITHUB_TOKEN=... specsync import github 42 --repo owner/repo
GITHUB_TOKEN=... specsync import --all-issues --repo owner/repo
```
