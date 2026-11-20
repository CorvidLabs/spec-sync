---
change: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths
artifact: tasks
---

# Tasks

- [x] Route or mark each scan site in the 17 files. Evidence: `ba597f66`, `87d2b356`; the read, write, Git and spawn scans pass.
- [x] Green on macOS (verify lane) and Linux, through one-lane. Evidence: at `2639d058`, the macOS verify lane passed all 10 steps, and Linux had 3162 unit and 741 integration tests passing.
