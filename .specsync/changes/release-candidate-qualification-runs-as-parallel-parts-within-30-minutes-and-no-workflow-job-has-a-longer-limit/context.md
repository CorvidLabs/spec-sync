---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: context
---

# Context

`v6.4.2-rc.1` did not qualify: every test passed on macOS, and the `qualify` job was cancelled at its 75-minute limit during the release build. `v6.2.0-rc.1` had done the same at 45 minutes (#860, #861). `v6.4.2-rc.2` qualified at 78m32s on macOS under a 120-minute limit that #1162 added as a stopgap. Leif accepted that once and asked for the opposite direction: every lane within 30 minutes, a lane that grows fails at its limit, and a release candidate does not fail on time (SPEED-2.b.3, SPEED-2.b.4, SPEED-2.b.5). #1161 has the measurements.

What a session picking this up needs to know:

- The macOS qualification is the only hosted lane over 30 minutes. In rc.2 its lane step was 65m48s of tests, about 5 minutes of that compiling, then 8m13s of release race guards and 3m59s of release build. Ubuntu took 22m31s in all. CI's longest job is coverage at 22m24s.
- The pinned cargo-nextest 0.9.151 has `--partition slice:K/N`, which deals the whole list across all binaries round-robin. On main `778a111a` four slices are disjoint and cover all 4,187 tests (1,047, 1,047, 1,047 and 1,046). On the hosted per-test times of rc.1 the four slices need 15.6 to 17.0 minutes of test time on macOS, so about 22 to 23 minutes a job with the compile.
- GitHub runs at most five macOS jobs at once on this plan, so four slices and one release-build part start together.
- Starting fewer Git processes for each lifecycle operation is the other half of #1161. It is product code and is a separate change.
- SPEED-2.b.2 bounds this one: no test is dropped or weakened.
