---
kind: evidence
id: 2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19
aliases: []
claim: "On base 7b6e4779 with only the new tests added: 15 of 19 project-link integration tests and 3 unit tests fail. hooks install and uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import and migrate write, create or rewrite files outside the project through a committed link; the resolve --remote cache is written and read through one."
result: failed
evidence_kind: product-test
method: "cargo test --locked --test integration -- project_links:: (4 passed, 15 failed); cargo test --locked --bin specsync -- spec_cache_ import_item_ (3 passed, 3 failed); rustc 1.89.0, isolated git config"
revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
content: 5e08c501f05cbabe08433b357bd742b93ef58d8d3385492d7c33e36cf0889fbf
environment: local-macos-arm64
criteria: [SETUP-6]
requirements: []
contracts: []
limitations:
  - "The 4 passing integration tests are the controls: a link that stays inside the project, a plain lock and a plain migration. The wizard is interactive and has no test on base; generator and registry are shown through scaffold. Requirement ids not linked with --requirement for modules with an underscore in their name."
observed: 2026-11-19T05:05:08Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On base 7b6e4779 with only the new tests added: 15 of 19 project-link integration tests and 3 unit tests fail. hooks install and uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import and migrate write, create or rewrite files outside the project through a committed link; the resolve --remote cache is written and read through one.**

Recorded as failed in local-macos-arm64 at revision 7b6e4779ba5b, observed 2026-11-19T05:05:08Z.

## Key failing lines

- hooks install --claude wrote through CLAUDE.md to a file outside the project; hooks install created the target of a dangling CLAUDE.md link; hooks install --copilot wrote into a directory outside the project; hooks uninstall rewrote a file outside the project through CLAUDE.md; agents install created the target of a dangling artifact link; agents install created the target of a dangling manifest link; agents install wrote into a directory outside the project; mcp lock --write wrote the lock through a link, over a file outside the project; new created the target of a dangling spec link; new wrote into a directory outside the project; scaffold created the target of a dangling companion link; scaffold appended to a registry outside the project; migrate created the target of a dangling .specsync/.gitignore link; migrate wrote into a directory outside the project; migrate rewrote a spec outside the project through a link
