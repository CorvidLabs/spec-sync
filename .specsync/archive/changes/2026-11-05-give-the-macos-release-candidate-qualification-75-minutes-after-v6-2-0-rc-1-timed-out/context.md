---
change: give-the-macos-release-candidate-qualification-75-minutes-after-v6-2-0-rc-1-timed-out
artifact: context
---

# Context

v6.2.0-rc.1 qualification (run 41971049002) passed on ubuntu in 11 minutes. On macos-15 every test passed (2764 unit tests in about 33 minutes, 478 integration tests in about 5.5 minutes), but the job hit its 45-minute `timeout-minutes` during the release `build` task and was cancelled. macOS qualification has grown from 28 minutes (6.1.0, 6.1.1) to 39 (6.1.2) to over 45. 0xLeif chose to raise the timeout for now and address the slowness separately.
