## ADDED

### REQUIREMENT REQ-change-172

When SpecSync reads an object of the repository's history in a partial clone and the object is not present, it SHALL first download every missing object that read needs in one fetch from the repository's promisor remote, run with the user's own Git environment and configuration and without prompting, and then read it as before. When that download fails, it SHALL refuse, saying the clone is partial and naming one `git fetch` command that downloads those objects.

Acceptance Criteria
- In a blobless clone of a repository whose promisor remote answers only with the user's configuration, `change review` of a change created in an earlier commit succeeds, and the objects it read are present afterwards.
- The download is one fetch, whatever the number of objects.
- When the remote cannot be reached, the refusal says the clone is partial and names a `git fetch … <oid> …` command that, run as printed, lets the same command succeed.
- A full clone makes no fetch.
- The fetch is one call bounded at 60 seconds. It never prompts: with no terminal, a credential that needs typing fails it, and the command then refuses as in option 2. Before it runs, one line on standard error says how many objects it downloads and from which remote.
- `change status`, `change audit` and `change ship-status` in a partial clone fetch the same way. A full clone never fetches.
