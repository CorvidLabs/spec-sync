---
change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
artifact: context
---

# Context

Issue #1077. In a blobless clone of a private HTTPS repository, SpecSync's isolated read of a historical object fails: `could not read Username for 'https://github.com': terminal prompts disabled` / `could not fetch <oid> from promisor remote`. The credential helper lives in the user's global or system configuration, which the isolated read switches off. SSH and public HTTPS clones read fine. One batched fetch with the user's configuration downloaded peck's 630 missing objects in 2 seconds.

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 1 and REQ-change-172, option 1. Leif's choice 1 in the tester's dialog of 2026-11-20 at about 00:45Z, "Fetch it itself (Recommended)", and "yes to all 3" at about 01:00Z: every command that reads history fetches, CI included. ss-lead's note: the fetch covers #1081's reads at the reference and at a change's base commit.
