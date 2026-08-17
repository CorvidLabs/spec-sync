## ADDED

### REQUIREMENT REQ-scoring-005

A spec whose `files:` entry is a directory SHALL score zero and name the directory, rather than scoring as a merely incomplete spec.

Acceptance Criteria
- The freshness dimension fails and names the directory, because a directory is not an existing source file.
- The API dimension is zero and names the directory, rather than reporting the path as missing or not valid UTF-8.
- The spec total is zero with grade F, which is below every strict and minimum-score floor including the inclusive eighty-point bar.
- Scoring remains a metric rather than a hard failure, so explain and machine-readable output still render for the affected spec.
- A spec naming a real source file is scored exactly as before, so the rule cannot be satisfied by lowering scores generally.
