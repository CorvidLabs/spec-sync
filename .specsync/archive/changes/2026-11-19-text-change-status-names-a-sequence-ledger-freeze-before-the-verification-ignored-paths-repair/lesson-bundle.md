# Lesson bundle: text-change-status-names-a-sequence-ledger-freeze-before-the-verification-ignored-paths-repair

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Text change status names a sequence-ledger freeze before the verification_ignored_paths repair
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, tests/integration/archived_upstream.rs
- **Acceptance**: with a frozen sequence ledger and an invalid verification_ignored_paths entry, text change status Next names the freeze, as JSON next_action does; without a freeze, text Next still names the verification_ignored_paths repair

## Evidence

- Verification commit: `7680a928ab6bb2782508a9272f027ed833790c99`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

#937, from a Codex P2 on #934 (thread PRRT_kwDORqhp_860ih0U), deferred by Leif's call.
`text_mode_next_action` (`src/commands/change.rs`) returned the `verification_ignored_paths`
repair before it checked for a sequence-ledger freeze. JSON `next_action` and the handoff name
the freeze first (REQ-change-138), so with a frozen ledger and an invalid ignored-path entry the
text `Next:` line named the policy repair while the text `Handoff:` line on the next row said
"clear the freeze named under Next". REQ-cmd-change-034 requires text status to match JSON.

The fix moves the freeze check above the ignored-path check. No requirement text changes. Part 2
of the original issue (the handoff carrying the ignored-path refusal) was fixed in #934.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-034 | `a_sequence_ledger_freeze_outranks_the_verification_ignored_paths_repair_in_text_status` (tests/integration/archived_upstream.rs) |
| REQ-change-138 | the same test asserts JSON `next_action` names the freeze, then the policy repair once the freeze is cleared |

## How it runs

`change check` runs only the in-process spec-code sync. The integration test runs locally with
`cargo test --test integration a_sequence_ledger_freeze_outranks` (red before the fix, green
after), and in hosted CI's full suite at the archived head.

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
