---
change: stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot
artifact: tasks
---

# Tasks

- [x] Remove the `Swatinem/rust-cache` step from the release `build` job
- [x] Replace it with a comment stating the job must never gain a caching step, and why `save-if: false` was insufficient here
- [x] Leave `qualify`'s cache untouched and record the open question rather than extending scope silently
- [x] Confirm `release.yml` still parses and no other caching step remains in `build`
- [x] Record in the CHANGELOG what `save-if: false` does and does not establish
- [x] Record that #63, #65 and #67 were dismissed as already mitigated and #68 was not
