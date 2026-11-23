---
change: check-fix-keeps-stdout-one-json-document-in-machine-formats
artifact: context
---

# Context

#475 (filed against 5.2.0). Reproduction 1, `init --json`, was confirmed fixed on 2026-11-10.
Reproduction 2 still held: `auto_fix_specs` (`src/commands/check.rs`) printed each spec's
`would add N export(s)` (dry run) or `added N export(s)` line on stdout in every format, so
`check --fix --json` and `check --strict --force --fix --dry-run --json` put human lines before
the JSON object. The caller already gates its own summary and backup lines on text mode, and the
existing `fix_with_json_output` test searched stdout for the first `{` to work around it.

cmd_check's acceptance criteria say JSON output is a single object. The issue asks for exactly
one parseable JSON document on stdout, with progress kept on stderr if it is kept. The fix keeps
the per-spec lines on stdout for text and prints them on stderr for every other format, the same
stream `--fix` failures already use in every format. No requirement text changes.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.
