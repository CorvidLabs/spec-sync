---
change: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
artifact: context
---

# Context

spec-sync#946, a deferred Codex P2 on #943. `upstream_archive_of` guards against archiving a change a second time when the remote default already holds its archive (REQ-change-129, #889; matched by state record since REQ-change-139, #907). Every Git failure on that path ends in `None`: a failed or truncated `ls-tree`, a failed `cat-file --batch` (for example past the 64 MiB `MAX_GIT_COMMAND_OUTPUT_BYTES` cap, or a missing object in a partial or damaged clone), an unparsable batch header, or a failed `log`/`merge-base` for a matching package. `None` means "no upstream archive", so review, status and finalize carry on and can archive a stale copy again.

Human intent: SHIP-3.c, captured from Leif on 2026-11-06 ("Capture as written"): "When the tool cannot tell whether my change was already archived on the main branch, review and finalize stop and tell me how to find out, instead of archiving it a second time."

Reproduced on a6116e0c (#943's archive tip; this branch stacks on #943 until it merges) by removing the archived `state.json` blob from the clone: `cat-file --batch` answers `missing`, the header parse fails, and `upstream_archive_of` returns `None`; `change review` then accepts.

Scope decisions:
- No remote default (`remote_default_ref` is `None`) stays "nothing to report": there is no main branch to compare with.
- An archived `state.json` that reads but does not parse as a change record is still skipped, as before. It is not this change's record, and refusing on it would block every change in the project.
- Reading the batch in bounded chunks, so a very large archive can be read instead of refused, is not in this change. Past the cap the guard now refuses with a recovery rather than switching off.
