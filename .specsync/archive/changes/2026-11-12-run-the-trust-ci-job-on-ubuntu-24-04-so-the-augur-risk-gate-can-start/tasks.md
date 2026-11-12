---
change: run-the-trust-ci-job-on-ubuntu-24-04-so-the-augur-risk-gate-can-start
artifact: tasks
---

# Tasks

- [x] Pin the trust job to `runs-on: ubuntu-24.04` with a comment citing
      CorvidLabs/augur#59 and the revert condition
- [x] File the upstream issue (CorvidLabs/augur#59: Linux release binary needs
      libxml2.so.2; fails on ubuntu-26.04's libxml2.so.16)
