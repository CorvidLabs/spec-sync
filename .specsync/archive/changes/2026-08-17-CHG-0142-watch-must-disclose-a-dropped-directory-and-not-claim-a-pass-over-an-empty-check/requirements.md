---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: requirements
---

# Requirements

## REQ-watch-002 — a configured directory that is not watched must be named

`watch` SHALL report every configured `specs_dir` or `source_dirs` entry that
does not exist, naming the path and its role, before it begins watching.

Acceptance
- The report names the configured path as written in the config, not the
  absolute path it resolved to.
- It states the role, so an operator with several source dirs knows which
  setting to correct.
- It appears in both the human and JSON output modes, on stderr, so the stdout
  banner remains machine-readable.
- A missing directory remains non-fatal while at least one directory exists.

## REQ-cli-009 — a pass must not be reported over a check that examined nothing

`watch` SHALL report a pass only when the check it ran examined at least one
spec. When the check found no specs, `watch` SHALL say that nothing was
checked.

Acceptance
- A tree whose `specs_dir` does not exist produces neither `All checks passed!`
  nor a claim of success.
- A tree whose specs exist and pass still produces `All checks passed!`.
- The check command's own exit codes and output are unchanged.
