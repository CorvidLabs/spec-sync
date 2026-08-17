---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: context
---

# Context

`change check` recorded `passed: true` and printed `✓ verified` while executing none of the four
verification commands the project configured.

Measured on this repository, two real changes minutes apart, same binary, same config:

    CHG-0137  --spec validator --spec manifest
      commands: ["cargo test validator::tests::"]
      63 tests; integration binary 0 passed / 400 filtered out
      rc=0  "✓ verified"

    CHG-0138  --no-spec-change, no --spec at all, affected_specs = []
      commands: ["ruby --version", "cargo test",
                 "python3 -S .github/scripts/validate-release-version.py",
                 "python3 -S .github/scripts/validate-workflow-runtime-pins.py"]
      rc=0  "✓ verified"

The change that declared **two real modules** ran one filtered command and none of its own proving
tests. The change that declared **nothing** ran the complete suite. `ruby --version` executed for
the first time in this repository's history, because it is reachable only through the fallback.

## Mechanism

`verification_commands_for_change`:

    for module in &record.affected_specs {
        if let Some(component) = routing.component_verification_commands.get(module) {
            commands.extend(component.iter().cloned());
        }
    }
    if commands.is_empty() {            // <- per CHANGE, not per module
        commands.extend(policy.verification_commands.iter().cloned());
    }

`commands.is_empty()` is evaluated once for the whole change. One routed module makes it false, so
the project-wide list is suppressed for **every** module in that scope — including the ones nobody
routed, which then contribute nothing at all and raise no warning.

Component commands **replace** the project list rather than adding to it, which is the opposite of
what the name suggests and of what the living spec already requires: `REQ-change-015` states
"Reporting mode still executes every configured verification command."

## Why it stayed hidden

The three routing keys — `component_verification_commands`, `strict_verification_commands`,
`strict_paths` — are top-level siblings of the policy fields, read by a **second, separate
deserialization** of `.specsync/sdd.json` (`load_verification_routing`). They are not part of
`SddPolicy`, so nothing that validates the policy ever sees them. `specsync init` never writes
them. `grep` across `specs/ docs/ site/ README.md AGENTS.md CHANGELOG.md` returns zero hits, and no
test exercised the component map returning `Some`.

A shadow config with no schema, no documentation and no coverage is exactly the substrate this
defect needed. Introduced 2026-07-31 in `32c8b350` (#480).

## Scoped out, deliberately

Zero-match detection — a `cargo test` filter selecting no tests exits 0, indistinguishable from a
filter that matched and passed. Catching it requires capturing command output, which
`REQ-change-058` forbids: "No lifecycle entry point suppresses configured verification command
output; every invocation inherits the parent streams." Doing it properly needs a tee and its own
requirement change. Tracked on #617.

This change also does not repair the ten archived records that already carry narrowed evidence;
the fix is forward-looking.
