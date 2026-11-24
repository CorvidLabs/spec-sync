---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-093

The lifecycle SHALL compute a handoff readiness for every change so an agent can tell whether
clearing its context — or handing the change to a fresh session — loses anything the lifecycle
still needs, and SHALL say what to do first when it does.

Acceptance Criteria
- `handoff_summary` returns `safe`, `conditional`, or `not-yet` with a plain-language reason, the
  resume command `specsync change status <id>`, and, when readiness is not `safe`, at least one
  concrete step to take before clearing. No reason or step contains a digest.
- `classify_handoff` is a pure function of `HandoffSignals`; every branch has a unit test that
  needs no repository.
- A frozen sequence ledger, a definition changed after its approval, an invalid correction
  ledger, stale legacy terminal evidence, and an archive whose terminal evidence is
  `corrupt-history` are `not-yet`, and the step named is the repair the lifecycle already requires
  (clear the freeze, re-approve, restore the ledger, reopen, restore the archive from trusted
  history).
- A Draft is never `safe`: open questions, stub artifacts, and a complete-but-unapproved
  definition are each `conditional`, and the steps name answering, finishing the artifacts, or
  approving — approval is the first boundary a fresh session can resume from.
- Uncommitted edits under the change's `affected_paths` make an approved, implementing, or
  verifying change `conditional` and name committing or writing the intent into `change.md`;
  uncommitted files under `.specsync/` alone never do, because `change review` then
  `change finalize` runs with that evidence uncommitted by design.
- A verifying change whose recorded verification is stale is `conditional` and names
  `specsync change check <id> --commit`; one whose verification is current is `safe` whether the
  scoped review has been recorded yet or not, and the reason says which step a fresh session
  resumes at.
- An accepted workflow-v2 change and an archived change whose terminal evidence authenticates
  (including `historical-transition-unproven`) are `safe`.
- For an archived change whose terminal evidence is `corrupt-history`, the step before clearing
  and the summary's next action are one string ending with `specsync change status -- <id>`:
  `git restore --source=HEAD --staged --worktree -- :(literal)<package>` when the package's tracked
  files differ from `HEAD`; the same restore from the commit that archived the package (named by
  `git log -1 --diff-filter=A --format=%H -- :(literal)<package>/state.json`) followed by a commit
  when the package matches `HEAD` but not that commit; and, when the package still matches the
  commit that archived it or no package or repository can answer,
  `specsync change status --json -- <id>`
  to read what `summary.terminal_evidence.reason` names and restore that from trusted Git history, since
  a package restore would change nothing. Running the named step through a shell on a tampered
  archive returns `authenticated-history` and `safe`.
- Every change ID and path interpolated into a printed command is quoted as one shell word, so a
  change ID or archive directory carrying shell metacharacters cannot turn the copied step into a
  different command. Each archive path is a literal Git pathspec, in the printed step and in the
  probes that choose it, so `*`, `?` and `[` in a package path match only that package. Each
  printed `specsync` command puts `--` before the change ID, so an ID that begins with `-` is read
  as the ID, not as a flag.
- A package path that holds a control byte, such as a newline or an escape, is printed in `$'…'`
  quoting with each control byte written as an escape. The printed step then holds no control
  byte and cannot add lines or terminal commands to the output (CHANGE-7.b), and run through bash
  it still restores that exact directory.
- `handoff_summary` judges an archived record from the same terminal-evidence verdict
  `ChangeSummary` carries, so the text line cannot call an archive `safe` that JSON reports as
  `corrupt-history`.
- `ChangeSummary` carries the decision as `handoff`, so JSON consumers read the same verdict the
  text line prints.
