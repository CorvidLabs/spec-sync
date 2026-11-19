---
change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
artifact: context
---

# Context

After `never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound` was archived, `specsync change audit` reported six meaningful changed paths that no change covers. Change 3's commit had updated their call sites, but its declared paths did not list them. Each is a one-line edit that passes the project root to a function change 3 gave a root parameter. Finalize did not catch it; the archive-first CI audit would.

Human decision, relayed by the planning session (2026-11-19, about 15:50Z): "yes", covering both this change's scope approval and its implementation review. The code was committed and reviewed with change 3.
