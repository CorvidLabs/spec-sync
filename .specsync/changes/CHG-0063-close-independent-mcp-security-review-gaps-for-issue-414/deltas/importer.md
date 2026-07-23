## MODIFIED

### REQUIREMENT REQ-importer-001

The importer SHALL normalize supported external content into safe local spec drafts while
sanitizing paths, secrets, markup, and oversized input.

Acceptance Criteria

- GitHub imports require explicit `GITHUB_TOKEN` and use the shared typed, bounded in-process REST
  path.
- GitHub imports revalidate repository access after an ambiguous issue 404 and never launch a
  `gh issue view` subprocess.
- GitHub issue titles that normalize to an empty safe module name are rejected before an imported
  item or output path is constructed.
- Jira and Confluence behavior remains unchanged.

### SPEC SECTION Invariants

1. `import_github_issue` delegates to `github::fetch_issue_details`.
2. Missing tokens, malformed payloads, transport failures, timeouts, and inaccessible repositories
   are errors rather than partial imported items.
3. The issue identity, title, body, labels, state, and URL are parsed through the shared typed GitHub
   response contract.
4. `gh` remains outside the importer read path.
5. Imported GitHub module names are nonempty after safe-name normalization.
