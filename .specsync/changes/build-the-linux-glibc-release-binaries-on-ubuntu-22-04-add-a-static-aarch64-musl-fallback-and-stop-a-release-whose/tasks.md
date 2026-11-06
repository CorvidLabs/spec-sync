---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: tasks
---

# Tasks

- [ ] `linux-baseline.py` check script and `test-linux-baseline.py` (written first; red evidence recorded)
- [ ] `release.yml` and `rc-assets.yml`: Ubuntu 22.04 gnu runners, aarch64 musl row, check before upload
- [ ] `ci.yml` `linux-baseline` job with the v6.2.0 negative control, gated and summarized
- [ ] `validate-release-candidate.py` expects `specsync-linux-aarch64-musl`; tests follow
- [ ] `linux-baseline-test` fledge task in the verify, ci and repo lanes
- [ ] `test-required-ci-gate.py` expects `linux-baseline` skipped on archive-only, review-only and site-only PRs
- [ ] `site/src/content/docs/quickstart.md` Install section names the floor and the musl fallback
- [ ] README, Action docs, RELEASING.md, HLD.md, SCOPE.md and CHANGELOG
- [ ] `github` spec: REQ-github-025 materialized by `change check`, version bump, Change Log row, and a lesson plane in `github.3md`
- [ ] Green local evidence: new-recipe ARM64 archives pass the check; all script tests and lanes pass

Human gates, not tasks: scope approval and implementation review, both relayed through orc t18.
