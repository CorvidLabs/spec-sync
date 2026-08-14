---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: plan
---

# Plan

Land before #576 and #572. Both touch coverage rendering, and the compiler
error this change introduces is what catches their re-implementations — #576
already wrote one. Landing this first converts a review question into a build
failure.
