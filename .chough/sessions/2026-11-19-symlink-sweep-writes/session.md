---
kind: session
id: 2026-11-19-symlink-sweep-writes
aliases: [symlink-sweep-writes]
objective: No install, scaffold, import, lifecycle, migrate, lock or remote-cache command writes through a symbolic link it finds in the repository (private sweep, handled with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules:
  - util
  - hooks
  - agents
  - mcp
  - registry
  - cmd_resolve
  - cmd_migrate
  - generator
  - cmd_scaffold
  - cmd_new
  - cmd_wizard
  - cmd_import
  - cmd_lifecycle
stage: build
stage_status: active
iteration: 1
callers: []
native_change: never-write-a-project-file-through-a-symbolic-link
pr: ""
criteria: [SETUP-6]
requirements: []
cases: []
design: []
files: [src/util.rs, tests/integration/project_links.rs]
observations: [2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19]
bugs: [2026-11-19-install-scaffold-import-and-migrate-commands-write-through]
blockers: []
waiting_on_human: []
run_consents: []
next: Read the verify lane and the Linux container results, commit, run chough change check for both changes, push to the private fork, update the PR, ask Leif for the two reviews through t4c
---
# No install, scaffold, import, lifecycle, migrate, lock or remote-cache command writes through a symbolic link it finds in the repository (private sweep, handled with GHSA-7q5v-8jcc-rx54)

## Goal

No install, scaffold, import, lifecycle, migrate, lock or remote-cache command writes through a symbolic link it finds in the repository (private sweep, handled with GHSA-7q5v-8jcc-rx54)

## Where it stands

- Failing tests first on 7b6e4779: 15 integration and 3 unit tests red, recorded as evidence and as a bug
- util project file functions on a cap-std directory handle; hooks, agents, mcp lock, registry and scaffold, new, add-spec, wizard, import, generator companions, migrate (with a preflight) and the resolve cache moved onto them; 12 specs, lessons, CHANGELOG and cli.md updated

## Decisions and why

- 0xLeif decided "Intent approval of proposed criterion SETUP-6": "Allow links that stay inside" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:53Z, asked whether to keep refusing every link or allow links that stay inside the repository. t4c put that answer into the wording 'A command I run in a repository somebody else wrote never writes outside it through a link it found there.' and relayed it over hc to t5r as his intent wording for SETUP-6); recorded by claude-code-t5r; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of never-write-a-project-file-through-a-symbolic-link": "Fold into the same private fix. Yes, fold them in. OK as proposed. Allow links that stay inside." (Four Claude Code chat answers to his planning session (hc t4c), each relayed by t4c over hc to t5r. 2026-11-18 evening MST: 'Fold into the same private fix', on the six other symlink-following writes the private sweep found. 2026-11-19 at about 04:50Z: 'Yes, fold them in', on the sibling spec-creating commands the builder found, and 'OK as proposed', on refusing with an error that names the path and skipping the resolve --remote cache with a stderr warning. 2026-11-19 at about 04:53Z: 'Allow links that stay inside', narrowing the refusal to links that leave the repository, dangling links and non-regular targets. lifecycle promote and demote, named among the siblings, were dropped after a test on 6.3.0 showed they are not affected); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve never-write-a-project-file-through-a-symbolic-link as 0xLeif
- agent choice (claude-code-t5r): put the shared functions in util beside confine_path_to_root, not in a new module, because a new module needs a new spec inside a change whose scope is already twelve modules
- agent choice (claude-code-t5r): hooks uninstall exits 1 only for a refused path, because every other uninstall error was a note with exit 0 before and the Claude Code settings notice must stay one
- agent choice (claude-code-t5r): a path that leaves the project by its own spelling (.. or another absolute path, such as a specs_dir configured outside the root) is refused too, because it reaches the same place a link would
- rejected: hardening lifecycle promote and demote because a test on released 6.3.0 showed discovery already refuses a linked spec file

## What was verified

- [On base 7b6e4779 with only the new tests added: 15 of 19 project-link integration tests and 3 unit tests fail. hooks install and uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import and migrate write, create or rewrite files outside the project through a committed link; the resolve --remote cache is written and read through one.](evidence/2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19.md)

## Open threads

## Timeline

- 2026-11-19T04:53:30Z · start · intent → intent · No install, scaffold, import, lifecycle, migrate, lock or remote-cache command writes through a symbolic link it finds in the repository (private sweep, handled with GHSA-7q5v-8jcc-rx54) ^t-a250dc
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - by: claude-code (hc t5r)
- 2026-11-19T04:54:49Z · gate-opened · intent → intent · Intent approval of proposed criterion SETUP-6 ^t-ebcef7
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - holds: intent
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T04:54:49Z · gate-closed · intent → intent · Intent approval of proposed criterion SETUP-6 ^t-3b1568
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - relayed_by:
    - decided_by: 0xLeif
    - words: Allow links that stay inside
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:53Z, asked whether to keep refusing every link or allow links that stay inside the repository. t4c put that answer into the wording 'A command I run in a repository somebody else wrote never writes outside it through a link it found there.' and relayed it over hc to t5r as his intent wording for SETUP-6
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T04:59:05Z · gate-opened · intent → intent · Scope approval of never-write-a-project-file-through-a-symbolic-link ^t-def9e9
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T04:59:05Z · gate-closed · intent → intent · Scope approval of never-write-a-project-file-through-a-symbolic-link ^t-269ab9
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - relayed_by:
    - decided_by: 0xLeif
    - words: Fold into the same private fix. Yes, fold them in. OK as proposed. Allow links that stay inside.
    - where: Four Claude Code chat answers to his planning session (hc t4c), each relayed by t4c over hc to t5r. 2026-11-18 evening MST: 'Fold into the same private fix', on the six other symlink-following writes the private sweep found. 2026-11-19 at about 04:50Z: 'Yes, fold them in', on the sibling spec-creating commands the builder found, and 'OK as proposed', on refusing with an error that names the path and skipping the resolve --remote cache with a stderr warning. 2026-11-19 at about 04:53Z: 'Allow links that stay inside', narrowing the refusal to links that leave the repository, dangling links and non-regular targets. lifecycle promote and demote, named among the siblings, were dropped after a test on 6.3.0 showed they are not affected
    - recorded_by: claude-code-t5r
    - native: specsync change approve never-write-a-project-file-through-a-symbolic-link
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T04:59:12Z · advance · intent → design ·  ^t-a630f1
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - by: claude-code (hc t5r)
- 2026-11-19T04:59:12Z · skip · design → plan · CLI and library change with no visuals or interactions ^t-174d1a
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - by: claude-code (hc t5r)
- 2026-11-19T04:59:14Z · advance · plan → build · native:approved ^t-0e9a46
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - gate: native:approved
  - revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
  - by: claude-code (hc t5r)
