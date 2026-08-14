---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: requirements
---

# Requirements

## REQ-types-007

Configuration SHALL record when it is the built-in defaults standing in for a config file
that exists but could not be loaded.

Acceptance Criteria
- A successful load records no such condition.
- A config file that cannot be read, and one that cannot be parsed, both record it, naming the file.
- The absence of a config file is not recorded as a failure, because the defaults are then the intended configuration.

## REQ-config-009

Loading SHALL distinguish an absent config file from one that exists and could not be
loaded.

Acceptance Criteria
- An absent file yields the built-in defaults with no recorded failure.
- An unreadable file yields the built-in defaults with the failure recorded.
- A file that fails to parse yields the built-in defaults with the failure recorded.

## REQ-validator-013

Retained configuration discovery SHALL record a parse failure rather than returning defaults
indistinguishable from a successful load.

Acceptance Criteria
- A config file that fails to parse during discovery records the failure alongside the defaults it fell back to.
- Source directory discovery is unaffected.

## REQ-commands-009

No command SHALL report a verdict derived from configuration that failed to load.

Acceptance Criteria
- A command that reads specs refuses to run when the configuration records a load failure, and names the file.
- The refusal states how to proceed: fix the file, or remove it to use the built-in defaults deliberately.
- A project with a valid configuration, and a project with none, are both unaffected.
- The refusal is applied once at the shared entry point, so no command can omit it.
