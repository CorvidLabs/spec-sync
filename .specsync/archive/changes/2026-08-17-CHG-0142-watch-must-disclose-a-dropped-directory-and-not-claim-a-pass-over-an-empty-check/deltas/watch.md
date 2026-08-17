## ADDED

### REQUIREMENT REQ-watch-002

`watch` SHALL disclose every configured directory it is not watching, and SHALL NOT report a pass over a check that examined no specs.

Acceptance Criteria
- A configured `specs_dir` or `source_dirs` entry that does not exist is reported before watching begins, naming the configured path as written in the config rather than the absolute path it resolved to.
- The report states the role, so a project with several source directories identifies which setting is wrong.
- The disclosure is emitted in both the human and JSON output modes, on stderr, so the stdout banner remains machine-readable.
- A missing directory remains non-fatal while at least one configured directory exists, because watch is a long-running development loop.
- An empty watch set still fails closed.
- A pass is reported only on positive evidence that the check examined at least one spec; when the check reports finding no specs, watch states that nothing was checked.
- A spec set that exists and passes still reports a pass, so the rule cannot be satisfied by never reporting one.
