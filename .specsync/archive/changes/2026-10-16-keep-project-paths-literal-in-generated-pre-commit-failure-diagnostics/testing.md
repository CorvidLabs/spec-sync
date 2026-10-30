---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: testing
---

# Testing

The five Unix installed-hook unit cases execute generated shell hooks using a disposable project and a fake checker that records exact argv. Both checker exit 0 and exit 1 are exercised. Assertions cover literal root, hook status, failure message and absence of shell marker effects. The baseline produced 2 passing and 3 failing cases; patched `cargo test hooks::tests` passed all 63 hook tests. The first full lane passed 2,586 tests but a separate watch Unix-socket fixture was denied by the local sandbox. The permitted full rerun passed all 2,587 unit tests, 462 integration tests, format/lint/types, release build, strict specs and release/required-CI fixture controls: receipt 20261016T033019Z-S4-fbca94, exit 0, 830.914 seconds. The newly built release candidate passed all 10 installed-hook reproduction cases, including exact roots and exit codes with no marker execution. Latest released SpecSync 6.1.1 strict/100% checks, all 62 spec quality floors (80), and the mandatory pre-push gate passed (receipt 20261016T033533Z-S4-8f6404). Cached documentation checks passed 23 tests, Astro diagnostics with zero errors, and a 43-page build (receipt 20261016T034800Z-S4-88f14f). The original draft is published as PR #818. Leif gave genuine scope approval on 2026-10-16; current qualification and implementation review/finalization remain separate evidence-bound steps. Exact-current-commit Trust and hosted review/CI remain publication gates. No human inspection, Linux qualification or released fix is claimed.

## Current lifecycle follow-up

The original product source and test bytes remain unchanged from the independently reviewed and locally qualified candidate. The genuine scope approval removes the absence-of-permission condition only; no new local test run, human inspection, native review/finalization or passing hosted gate is inferred from that approval. Fresh command results will be recorded after execution.
