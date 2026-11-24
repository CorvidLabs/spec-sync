---
change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
artifact: context
---

# Context

#990, a Codex P2 on #988 deferred by Leif's call (P2 policy: issue, reply, resolve).
`workflow_v1_record_present_in_cutoff_tree` (`src/change/validate_loaded_change.rs`) first reads
`<cutoff>:<archive root>`. When that fails it probes `ls-tree --full-tree -z --name-only <cutoff>
-- <archive root>` to tell "the cutoff has no archive root" (absent) from "the archive tree is
there but unreadable" (an error, REQ-change-145). The probe passed the repository-relative path
as a plain pathspec. For a project in a directory whose name begins with pathspec magic, such as
`:(exclude)app`, `ls-tree` rejects the magic and fails, so a cutoff with no archive root was
reported as "could not be read". That fails closed, but it is the wrong verdict.

The fix passes `:(top,literal)<archive root>`, as the module's other history queries do (#851),
and keeps comparing listed entries with the plain path. The unreadable-tree error path is
unchanged (`an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change`).

The old `0xleif/fix/g-literal-929-930-931-990` WIP branch (2026-11-06) is stale; this is a fresh
change from main for #990 alone.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.
