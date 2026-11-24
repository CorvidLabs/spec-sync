---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: context
---

# Context

Three Codex P2s on #901 (the corrupt-archive restore step, CHANGE-7.a.2), deferred by Leif's call
and filed as #929, #930 and #931. All three are in `corrupt_archive_recovery` and the probes in
`corrupt_archive_signal` (`src/change/historical_definition_digest_matches.rs`). The coordinator
(t370) asked for one PR that closes all three, because they edit the same function.

- #929: `validate_change_id` accepts `*` and `?`, so a package path can hold them. Shell quoting
  stops the shell from expanding them, but Git still reads the path as a pathspec glob. The printed
  `git restore -- '<package>'` can then restore sibling archives and overwrite their uncommitted
  edits, and the `git diff --quiet` and `git log --diff-filter=A` probes are ambiguous the same way.
- #930: an id that begins with `-` loads, but the printed `specsync change status '<id>'` hands it
  to Clap as a flag, so the confirmation step reads no archive.
- #931: archive directories are found by the id in their `state.json`, so a directory name is
  never validated. A newline or ESC byte in it reached the printed `Next:` and `Handoff:` lines
  raw, inside the shell quotes.

#929 and #930 break REQ-change-093 as it stands ("cannot turn the copied step into a different
command"; "running the named step ... returns authenticated-history"). #931 has no requirement
yet, so it adds one acceptance line under the HI criterion CHANGE-7.b. Leif approved that sentence
in the orc tab on 2026-11-06 ("Capture both"); it sat on the unmerged g-literal branch and is
carried here unchanged. The red tests are also carried over from that branch.

Out of scope, and filed as a follow-up: other next actions that interpolate an id (for example
the plain `specsync change status <id>` resume line) do not put `--` before it either.

Chough: Leif's rule (relayed by t370 on 2026-11-13) is "make sure u use chough 0.6.0 for all
spec-sync changes". spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges, so red
tests and the fix are built now, and this change goes through a Chough 0.6.0 session after main is
merged in, before review.
