---
change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
artifact: tasks
---

# Tasks

- [x] Move the musl-tools install into .github/scripts/install-musl-tools.sh with apt retries, timeouts and the mirror fallback
- [x] Keep the per-target env exports in the ci.yml step unchanged
- [x] Add .github/scripts/test-install-musl-tools.sh against a stubbed sudo, dpkg and apt-get
- [x] Run the test in validate-action and as the fledge musl-install-test task
