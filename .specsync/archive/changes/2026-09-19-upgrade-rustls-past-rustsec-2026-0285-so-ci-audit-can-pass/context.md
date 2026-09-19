---
change: upgrade-rustls-past-rustsec-2026-0285-so-ci-audit-can-pass
artifact: context
---

# Context

PR #788 only adds a Human intent block to `AGENTS.md`. That path is not
meaningful, so the lifecycle gate was green, but `classify` still selected a
full CI run. `cargo audit` then failed on rustls 0.23.38,
RUSTSEC-2026-0285 (TLS 1.3 handshake messages accepted across encryption
level boundaries; fix is >=0.23.45, dated 2026-09-14). Main last went green
on 2026-09-12, before the advisory.

`Cargo.lock` is a meaningful path, so this bump needs its own change record.
Do not ignore the advisory. The remaining `instant` unmaintained warning is
already allowed.

spec-sync does not depend on rustls directly; `ureq` does. `cargo update -p rustls`
also moved `rustls-webpki` 0.103.13 → 0.103.15.
