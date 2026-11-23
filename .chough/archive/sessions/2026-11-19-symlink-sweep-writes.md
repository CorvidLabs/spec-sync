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
stage: done
stage_status: archived
iteration: 1
native_change: never-write-a-project-file-through-a-symbolic-link
criteria: [SETUP-6]
files: [src/util.rs, tests/integration/project_links.rs]
observations:
  - 2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19
  - 2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests
  - 2026-11-19-the-full-verify-lane-passes-on-the-code-of-the-pushed-tip-with
bugs: [2026-11-19-install-scaffold-import-and-migrate-commands-write-through]
title: Project file writes never leave the project
summary:
  - Install, scaffold, import, lifecycle-free spec creation, migrate, MCP lock and remote-cache commands wrote fixed project paths with writes that follow symbolic links, so a cloned repository could make them change or create files outside it. Found by the private sweep after GHSA-7q5v-8jcc-rx54.
  - "util now reads and writes through a cap-std handle on the project root: a link inside the project is followed, a link out, a dangling link or a non-regular target is refused naming the path; migrate refuses before its first step; the remote cache refuses any link."
  - "Verified: 15 integration and 3 unit tests failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: advisory text, then publishing and release on Leif's yes."
closed: 2026-11-19
outcome: done
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
- 0xLeif decided "Implementation review of never-write-a-project-file-through-a-symbolic-link": "Approve" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:04Z, after t4c walked him through the change in plain words from its own read of the code at a33dcc6f, with its side effects and limits (hooks uninstall exits 1 on a refused path, a specs_dir outside the project is refused, migrate refuses before its first step, hard links not caught, no x86 or Ubuntu run); relayed by t4c over hc to t5r as his implementation approval); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change review never-write-a-project-file-through-a-symbolic-link as 0xLeif with verdict pass

## What was verified

- On base 7b6e4779 with only the new tests added: 15 of 19 project-link integration tests and 3 unit tests fail. hooks install and uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import and migrate write, create or rewrite files outside the project through a committed link; the resolve --remote cache is written and read through one. ^2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19
- With the fix at a33dcc6f, all 19 project-link integration tests pass and the unit tests of every touched module pass (239). The same suite passes on Linux arm64 in a local container as a normal user: unit 3099 passed, integration 650 passed plus one test that needs jq and passes once jq is installed. ^2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests
- The full verify lane passes on the code of the pushed tip with both changes: unit suite 3095 passed, 0 failed, 2 ignored; integration suite 648 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the six gate scripts. ^2026-11-19-the-full-verify-lane-passes-on-the-code-of-the-pushed-tip-with

## Bugs

- [Install, scaffold, import and migrate commands write through symbolic links](../../bugs/2026-11-19-install-scaffold-import-and-migrate-commands-write-through.md): verified

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
- 2026-11-19T05:30:45Z · advance · build → test · native:verifying ^t-2ec0f8
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - gate: native:verifying
  - revision: a33dcc6ffd6210e4998c376edab1d331083bcd7f
  - by: claude-code (hc t5r)
- 2026-11-19T05:31:01Z · advance · test → accept · native:verifying ^t-124de7
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - gate: native:verifying
  - revision: a33dcc6ffd6210e4998c376edab1d331083bcd7f
  - by: claude-code (hc t5r)
- 2026-11-19T05:31:02Z · gate-opened · accept → accept · Implementation review of never-write-a-project-file-through-a-symbolic-link ^t-4ad045
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: a33dcc6ffd6210e4998c376edab1d331083bcd7f
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
- 2026-11-19T14:09:01Z · gate-closed · accept → accept · Implementation review of never-write-a-project-file-through-a-symbolic-link ^t-1b40f1
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - revision: eda76b6083162618fb01fe3592f86570f8334cea
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:04Z, after t4c walked him through the change in plain words from its own read of the code at a33dcc6f, with its side effects and limits (hooks uninstall exits 1 on a refused path, a specs_dir outside the project is refused, migrate refuses before its first step, hard links not caught, no x86 or Ubuntu run); relayed by t4c over hc to t5r as his implementation approval
    - recorded_by: claude-code-t5r
    - native: specsync change review never-write-a-project-file-through-a-symbolic-link
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T14:09:02Z · advance · accept → done · native:archived ^t-42152a
  - native_change: never-write-a-project-file-through-a-symbolic-link
  - gate: native:archived
  - revision: eda76b6083162618fb01fe3592f86570f8334cea
  - by: claude-code (hc t5r)

Closed 2026-11-19: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19
    claim: "On base 7b6e4779 with only the new tests added: 15 of 19 project-link integration tests and 3 unit tests fail. hooks install and uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import and migrate write, create or rewrite files outside the project through a committed link; the resolve --remote cache is written and read through one."
    result: failed
    revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
    environment: local-macos-arm64
    content: 5e08c501f05cbabe08433b357bd742b93ef58d8d3385492d7c33e36cf0889fbf
    observed: 2026-11-19T05:05:08Z
    actor: claude-code (hc t5r)
  - id: 2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests
    claim: "With the fix at a33dcc6f, all 19 project-link integration tests pass and the unit tests of every touched module pass (239). The same suite passes on Linux arm64 in a local container as a normal user: unit 3099 passed, integration 650 passed plus one test that needs jq and passes once jq is installed."
    result: passed
    revision: a33dcc6ffd6210e4998c376edab1d331083bcd7f
    environment: local-macos-arm64
    content: 832a247c578e6ffff491fae9f2cbe21f1e457b4c792e5da8b4254ec7becd3b64
    observed: 2026-11-19T05:30:29Z
    actor: claude-code (hc t5r)
  - id: 2026-11-19-the-full-verify-lane-passes-on-the-code-of-the-pushed-tip-with
    claim: "The full verify lane passes on the code of the pushed tip with both changes: unit suite 3095 passed, 0 failed, 2 ignored; integration suite 648 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the six gate scripts."
    result: passed
    revision: 3a18f83c7701b31854549fa595f5a32c5c6c4302
    environment: local-macos-arm64
    content: 832a247c578e6ffff491fae9f2cbe21f1e457b4c792e5da8b4254ec7becd3b64
    observed: 2026-11-19T05:53:55Z
    actor: claude-code (hc t5r)
```
