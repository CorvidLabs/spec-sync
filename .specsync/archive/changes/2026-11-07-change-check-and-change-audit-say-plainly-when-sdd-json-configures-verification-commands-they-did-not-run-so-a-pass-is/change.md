---
id: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
state: archived
type: feature
base_commit: 22a2dcaca598949dfb3a34907ea2fb0cfd18a820
---

# Change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)

## Intent

change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- Text change check (plain and --commit) and change audit print one stderr note beginning `note: .specsync/sdd.json configures N verification command(s) that SpecSync did not run`, with singular `command` for N=1 and the words `to stop this note`.
- JSON check and audit output carries a top-level configured_commands_not_run list beside verification or report: verification commands first, component commands by module, then strict commands, each command once, and [] when none is configured.
- The disclosure never changes exit status and never spawns a configured command; change check --commit --json gains no field because that path has no JSON success output (#839).
- This repository empties only verification_commands while retaining all 13 component_verification_commands modules and all 3 strict_verification_commands, so its own note remains visible.
- The committed text, JSON, singular, empty-policy, no-spawn, and repository-policy tests fail before implementation and pass afterward, with the full integration suite updated for the additive stderr note where needed.

## No-spec Rationale

Not applicable
