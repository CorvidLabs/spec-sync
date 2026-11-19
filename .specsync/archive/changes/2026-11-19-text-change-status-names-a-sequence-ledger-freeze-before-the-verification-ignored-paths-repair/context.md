---
change: text-change-status-names-a-sequence-ledger-freeze-before-the-verification-ignored-paths-repair
artifact: context
---

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
