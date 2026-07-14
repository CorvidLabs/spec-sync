---
change: CHG-0028-use-the-release-candidate-specsync-binary-in-the-trust-workflow
artifact: plan
---

# Plan

1. Build the release candidate with the locked Cargo dependency graph.
2. Package `specsync-linux-x86_64.tar.gz` beneath `runner.temp` using the archive layout required by the pinned action.
3. Generate the adjacent SHA-256 file and pin the verified Trust v1.0.1 commit.
4. Pass SpecSync version 5.0.2 and the runner-local mirror into the Trust action.
5. Validate workflow syntax and hosted Trust without disabling any gate.
