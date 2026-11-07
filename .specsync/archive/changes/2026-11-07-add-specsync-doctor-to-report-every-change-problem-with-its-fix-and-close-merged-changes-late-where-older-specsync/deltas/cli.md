## MODIFIED

### SPEC SECTION Purpose

The `specsync` command-line entry point parses global options, refuses `check`, `change`, or `lifecycle` commands that run under an inherited `SPECSYNC_VERIFICATION_CONTEXT` (SpecSync starts no verification child of its own), routes canonical validation, verified SDD lifecycle commands and `specsync doctor` (REQ-cmd-change-035) to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.
