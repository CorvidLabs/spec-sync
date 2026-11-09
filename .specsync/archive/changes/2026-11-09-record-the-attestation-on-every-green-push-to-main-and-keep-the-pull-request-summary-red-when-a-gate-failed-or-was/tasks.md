---
change: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
artifact: tasks
---

# Tasks

- [x] `attest`'s `if:` starts with `always()` and still requires `ci-gate` success, push and main
- [x] The summary rows carry `<label>=<selected>=<result>` with each job's own selection; `CHECK_PREFLIGHT` and `CHECK_LIFECYCLE` are added
- [x] The summary script reads `true:skipped` as blocked and red, and `false:skipped` as not selected
- [x] `ci-attest-and-summary-test` runs in the fledge `ci`, `verify` and `repo` lanes and in `ci.yml`'s `validate-action` job
- [x] `specs/github`: REQ-github-029 and REQ-github-030 materialized, a testing row and a `github.3md` lesson plane
