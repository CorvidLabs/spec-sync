## ADDED

### REQUIREMENT REQ-change-072

The change sequence ledger gate SHALL judge a ledger against the highest mark the current branch has itself recorded, and SHALL NOT refuse a branch for trailing the default branch.

Acceptance Criteria
- A branch whose ledger is older than the default branch's, but consistent with its own history, is accepted, and allocation on it continues to floor against the remote mark so it cannot remint an ordinal the default branch already used.
- A ledger below the highest mark the branch itself recorded is refused, including when the branch raised the ledger and then rewrote it downwards to a value still above the point at which it diverged.
- The gate consults no remote, so a repository without an origin is judged by the same rule rather than having the gate silently disabled.
- The refusal names the mark that was lost and a recovery command that applies to the branch's own history.
