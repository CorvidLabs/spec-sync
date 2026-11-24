---
change: give-the-macos-release-candidate-qualification-120-minutes
artifact: context
---

# Context

`v6.4.2-rc.1` did not qualify. Ubuntu passed in 15m59s. On macOS all 4,176 tests passed, but that step took 69m41s and the `qualify` job was cancelled at its 75-minute limit while `test-release-guards` was compiling. A complete macOS run needs about 84 minutes, so a re-run cannot fit. `v6.2.0-rc.1` was cancelled the same way at 45 minutes, and #860 raised the limit to 75.

The whole increase since 6.4.1 is the 277 tests this release added. The time is Git process starts: the slowest test starts 11,576 of them, at about 18.5 ms each on the hosted macOS runner and 2.8 ms on Ubuntu. #1161 has the measurements.

0xLeif decided to raise the limit to 120 and cut `v6.4.2-rc.2`, so the release is not held. He also said he does not want the limit raised again and wants every test lane to finish within about 30 minutes. That work is #1161. This change moves away from SPEED-2.b.1 on purpose and for now. SPEED-2.b.2 holds: no test is dropped or weakened.

Ruled out for today: re-running rc.1, which would time out again, and splitting the macOS run into parallel jobs, which changes the candidate evidence records and their validator and is not a change to make while a release waits.
