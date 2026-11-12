---
change: run-the-trust-ci-job-on-ubuntu-24-04-so-the-augur-risk-gate-can-start
artifact: context
---

# Context

GitHub is rolling `ubuntu-latest` over to `ubuntu-26.04`. On 2026-11-11 this
repository's `trust` job failed twice (run 43099930195, 21:08 and 21:17 UTC):
the CorvidLabs Trust gate's risk step downloads the prebuilt
`augur-linux-x86_64` binary, which links `libxml2.so.2` via Swift
FoundationXML, and 26.04 ships libxml2 2.15 under the new SONAME
`libxml2.so.16` — there is no `libxml2.so.2` to install. The binary exits 127
before doing any work. The same workflow passed hours earlier on an
`ubuntu-24.04` runner image (run 43092410912, 20:00 UTC), so the runner OS is
the only variable.

`apt install libxml2` was ruled out: no 26.04 package provides the old SONAME.
The durable fix is upstream (CorvidLabs/augur#59: a static or
libxml2.so.16-linked rebuild); until then, the trust job pins
`runs-on: ubuntu-24.04`, which still ships `libxml2.so.2`. The comment on the
pin names the issue and the revert condition.

Only the trust job changes. Every other job keeps its runner; the Archive job
in the same workflow does not run augur and stays on `ubuntu-latest`.
