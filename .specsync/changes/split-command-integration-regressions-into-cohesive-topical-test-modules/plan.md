---
change: split-command-integration-regressions-into-cohesive-topical-test-modules
artifact: plan
---

# Plan

## Approach

Keep tests/integration/commands.rs as the registered module entry point and move complete tests into meaningful command/domain groups beneath tests/integration/commands/. Group small related commands rather than create arbitrary tiny shards. Put shared process/platform/race fixtures at the nearest common parent with restricted visibility. Preserve original function names and complete bodies; only module-qualified paths change.

## Out of scope

No production source, public CLI/API, test assertions, fixture inputs, expected diagnostics, process environment, race barriers or security semantics change. Do not weaken or rewrite the existing root integration registration guard, delete tests, broaden platform execution or repair unrelated baseline failures. No publication, human implementation acceptance, archive or merge is implied.

## Steps

1. Preserve the pristine-main command test list and exact body/attribute inventory; verify active-PR clearance.
2. Extract issue/configuration/reference tests and confinement tests with shared platform fixtures.
3. Extract coverage/Gradle/input and retained-root race tests into cohesive owners, preserving subprocess filters and capability test setup.
4. Group generation/companions, project setup/hooks, history/diff, migration, dependency analysis and spec-maintenance regressions into topical modules below 1000 lines wherever coherent.
5. Retain the existing integration harness entry and explicit child declarations; ensure every relocated test is compiled and the original root registration guard still passes. Keep helpers private to their nearest common owner.
6. Document the layout in the CLI spec/companions. Existing references to commands.rs continue to point to its registered module entry point.
7. Compare original bodies/literals, test names and cfg attributes; run all command tests and the complete integration suite, compare known failure sets, run formatting and relevant lint/spec/pre-push checks, and obtain independent technical review.
8. Use the real scoped lifecycle materialize/verify commit. Preserve full-trust and platform-runtime limitations and require genuine later human review.

## Risks

An unregistered child module can silently drop tests, so compare the compiled inventory by original function name and retain the root harness entry. Relocated helpers may lose Windows/Unix guards or broaden visibility; preserve attributes and inspect platform-only fixture types. Child process test filters must still select exactly their intended test. Keep fixture bodies and environment sanitization unchanged.

## Constraints consulted

Read AGENTS.md, CLI requirements/context/testing, the command test file and integration harness registration guard. Honor exact CLI formats, exit codes, no-write assertions, environment cleanup and requested-root capability binding. Preserve all security/race regressions and platform/debug gates. hi/plan.md requires substantive structured planning; hi/agent.md forbids self-approved implementation review. No configured principles file exists.
