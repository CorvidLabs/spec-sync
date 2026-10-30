# Lesson bundle — advance-the-published-specsync-6-1-1-action-default-after-platform-consumer-checks

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the published SpecSync 6.1.1 Action default after platform consumer checks
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, CHANGELOG.md, README.md, action.yml, site/src/content/docs/integrations/github-action.md, specs/github/github.3md, specs/github/github.spec.md, specs/github/testing.md, site/src/content/docs/configuration.md
- **Acceptance**: After actual immutable-candidate Ubuntu/macOS qualification, validated stable v6.1.1 assets and successful immutable-A explicit-version Linux/macOS consumers, advance the separate main Action wrapper default, offline declaration and corresponding docs to published 6.1.1. Preserve the composite runtime script bytes and explicit CI/Trust candidate mirrors.
- **Acceptance**: Preserve immutable v6.1.1 and RC refs at qualified candidate e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa with embedded default 6.1.0 and explicit version 6.1.1 guidance; preserve the immutable v6.0.0 caveat. Represent the exact accepted Purpose with one normal github SpecSection delta, unchanged permanent REQ-github-002, actual current-parent canonical version and actual new change ID, preserving A and all historical rows.
- **Acceptance**: Remove only the obsolete prepared-release/published-version sentence from site/src/content/docs/configuration.md while preserving preceding reader/downgrade warnings and without adding a new runtime or distribution claim. Qualify the fresh exact B tree and normal native/CI/review/ship gates. Floating v6 may move only after omitted-input Linux/macOS consumers pass on the exact selected B target, without version override or immutable-tag movement.

## Evidence

- Verification commit: `5e5368fd3ac87a46685f1d6abec3918bd715342c`
- Base commit: `e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

Leif accepted the concrete tested Phase A and conditional eight-file Phase B with the direct response “I approve”; the controller human-acceptance.json binds the exact Phase B patch SHA256 `0f2816db8c73fa4b70fa541afa087d10b110614e8bfec71db9a5a3f2905f93ae`. This package must retain those bytes/behavior, adapting only actual parent-native version and change-ID bookkeeping.

Phase A is archived and integrated at immutable candidate `e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa`. The recorded release run `38752343005` qualified that same SHA on Ubuntu and macOS. This proves candidate qualification, not stable publication or packaged public-asset consumer behavior. The public wrapper at this immutable SHA still defaults to `6.1.0`.

Before copying these artifacts into a live native B package, resolve and verify these real activation receipts:

- Stable release/tag/complete asset and checksum validation: `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/validated-stable-publication.json (SHA256 `52fe7eb583d1b4c50e1f516f54f78a5b08019157b693cfd910125e8bfaeefad8`)`.
- Actual immutable-A explicit `version: '6.1.1'` Linux consumer: `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/linux-explicit-89x5vmnz/receipt.json (SHA256 `ee0e825f14c9b882a49f1178aad8739d36de8fa264c38c1fc8963cde3d084629`)`.
- Actual immutable-A explicit `version: '6.1.1'` macOS consumer: `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/macos-explicit-_3ek3u28/receipt.json (SHA256 `b0321aa51232236b24ca3b9918b1fb1c8c23301c388b76ebff3bab6274602993`)`.

All three actual activation receipts have passed and were checked during this native packet preparation. Their exact paths/hashes and execution limits are recorded below. Future B tree qualification, lifecycle approval, CI and omitted-input consumer results remain separate and incomplete.

Later B product/archive CI and omitted-input consumer evidence depend on an actual product SHA. Record them in the controller's external evidence ledger after native check without editing native execution artifacts or pretending future evidence existed at the check anchor.


## Recorded actual activation receipts

Stable publication/assets validation is complete: /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/validated-stable-publication.json (SHA256 `52fe7eb583d1b4c50e1f516f54f78a5b08019157b693cfd910125e8bfaeefad8`). The actual stable release `v6.1.1`, immutable RC and final tag resolve to candidate `e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa`; five public archives and five sidecars were matched to actual promotion artifacts/provenance. Dual-OS immutable qualification remains run `38752343005`.

Both explicit consumers are complete in /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/explicit-consumers.json (SHA256 `50b0f135ea07c124e56989ff9eaba7010e3fba2176322607cecbd22ef0f42897`). They executed the frozen immutable-A Action shell steps against real public assets: native macOS/aarch64 and cached Docker Linux/aarch64. This is literal Action-shell/process evidence, not full hosted GitHub composite-engine proof. No mirror or comments were used.

| Platform | Actual child receipt and hash | Stage exits |
|----------|-------------------------------|-------------|
| macos/aarch64 | /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/macos-explicit-_3ek3u28/receipt.json (SHA256 `b0321aa51232236b24ca3b9918b1fb1c8c23301c388b76ebff3bab6274602993`); wrapper receipt `152517-CODEX-958d07` | download/version/positive/coverage `0`; actual Action negative and negative JSON `1` |
| linux/aarch64 | /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/linux-explicit-89x5vmnz/receipt.json (SHA256 `ee0e825f14c9b882a49f1178aad8739d36de8fa264c38c1fc8963cde3d084629`); wrapper receipt `152538-CODEX-dc6b6b` | download/version/positive/coverage `0`; actual Action negative and negative JSON `1` |

Both children bind immutable wrapper/action SHA256 `5fefb55e785da35940eb8660884de1a6e057777a15f63dff9156b40896fc0b72`, download-step SHA256 `ce14d6db30f132ed1a4075c4deea4c0174c0339a6d6908305e5c8b99811ad61c` and check-step SHA256 `68121d7cab4c5108b86d585f9ea2fce0eeccc5a9ec9beb4ca597b2b993d326f7`. They supplied explicit `version: 6.1.1` while observing the unchanged embedded default `6.1.0`. Each receipt records exact public URLs, archive/sidecar/downloaded-binary hashes, logs and all real exits. These actual activation results do not qualify the new B source tree or its future omitted-input consumers.


## Factual ninth documentation path

The actual affected paths include `site/src/content/docs/configuration.md` in addition to the eight accepted B delivery paths. Root's ordinary factual correction removes only the obsolete two-line assertion that6.1.1 is still prepared and6.1.0 remains published. It preserves the preceding reader/downgrade warnings and adds no new runtime behavior, requirement or distribution claim. The accepted eight-file B behavior/Purpose is unchanged. No `specs/github/tasks.md` update is included in this package unless root actually records truthful existing progress through native scope first.

## From the change's design.md

# Design

Apply the accepted eight-file postpublication patch to the actual archived Phase A parent. Advance only `PUBLISHED_ACTION_DEFAULT`, Action input default/description and corresponding current docs/context/testing from `6.1.0` to `6.1.1`. The composite `runs` implementation, downloader/checksum/extraction/runtime behavior and explicit candidate CI/Trust mirrors stay unchanged. There is no dynamic selector, new dependency or CLI runtime change.

Use `deltas/github.md` verbatim: one MODIFIED SPEC SECTION Purpose, with the full accepted Purpose block and no new REQUIREMENT item. Keep permanent REQ-github-002 bytes and every unrelated canonical section. Preserve the accepted manual changelog row exactly. Generate the real B change ID and current-parent canonical version via supported native operations, retaining A's existing row/version history.

Stable binary identity remains the qualified A commit. B is a later wrapper identity, not a new CLI release or replacement immutable tag. Public A explicit-version consumer checks activate B; actual B omitted-input Linux/macOS consumers qualify floating movement. A failed gate causes no ref/default advancement and no change to protection rules.

Complete pre-check artifacts and attach known activation/local receipts before native check freezes the execution digest. Future product/CI/archive/omitted-consumer observations live in the external controller ledger; do not edit frozen native testing/tasks artifacts to backfill those later observations.


## Factual ninth documentation path

The actual affected paths include `site/src/content/docs/configuration.md` in addition to the eight accepted B delivery paths. Root's ordinary factual correction removes only the obsolete two-line assertion that6.1.1 is still prepared and6.1.0 remains published. It preserves the preceding reader/downgrade warnings and adds no new runtime behavior, requirement or distribution claim. The accepted eight-file B behavior/Purpose is unchanged. No `specs/github/tasks.md` update is included in this package unless root actually records truthful existing progress through native scope first.

## From the change's testing.md

# Testing

## Known evidence and its limits

Historical exact accepted external A/B prototypes passed the production version validator and all64 real release-validator tests in `131336-CODEX-efc001`. A's actual immutable candidate `e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa` qualified Ubuntu/macOS in run `38752343005`. Neither result substitutes for fresh B source/native qualification or real public asset/Action consumer checks.

## Activation evidence required before native B creation/check

| Gate | Actual receipt | Observed activation status |
|------|-------------------------|----------------------------------|
| Stable tag, complete archive+checksum assets, exact candidate identity | `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/validated-stable-publication.json (SHA256 `52fe7eb583d1b4c50e1f516f54f78a5b08019157b693cfd910125e8bfaeefad8`)` | PASS; actual stable/tag/assets validation inspected |
| Immutable-A explicit6.1.1 Linux consumer | `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/linux-explicit-89x5vmnz/receipt.json (SHA256 `ee0e825f14c9b882a49f1178aad8739d36de8fa264c38c1fc8963cde3d084629`)` | PASS; actual explicit public-asset shell/process consumer inspected |
| Immutable-A explicit6.1.1 macOS consumer | `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/macos-explicit-_3ek3u28/receipt.json (SHA256 `b0321aa51232236b24ca3b9918b1fb1c8c23301c388b76ebff3bab6274602993`)` | PASS; actual explicit public-asset shell/process consumer inspected |

## Fresh local qualification on the applied native B tree

Run the unchanged configured pinned `fledge lanes run verify`: fmt/lint/types, full Rust tests, optimized build, strict100% specs, `release-candidate-test` and `ci-gate-test`. The real release suite is64 tests in `.github/scripts/test-validate-release-candidate.py`; the real required-gate suite is29 controls in `.github/scripts/test-required-ci-gate.py`. Also run the actual production `.github/scripts/validate-release-version.py`, applicable current audit, latest forced strict checks and independent exact-tree review. Run mandatory pre-push and Trust/Attest later at their actual commit/publication boundaries; none is asserted complete here. Preserve real logs, counts, exit codes, tool versions and source hashes. Do not skip/relabel a failing step or substitute the prototype receipts.

## Manual existing-requirement traceability

| Existing requirement | Evidence and completion boundary |
|----------------------|----------------------------------|
| REQ-github-002 (permanent bytes unchanged) | Stable publication/assets and explicit A consumers activate accepted B. Fresh wrapper/validator/full native verification demonstrates consistent6.1.1 defaults/docs and preserved immutable-tag explicit input caveat. Actual exact-product CI and genuine native review+ship/finalCI plus real omitted-input Linux/macOS B consumers precede floatingv6 movement. Record each gate independently with exact SHA and real receipts. |

The delta is SPEC SECTION Purpose only. Native requirement_ids is expected to be empty; it does not claim the native scoped check executed this manual asset/platform matrix. Required manual evidence remains independent and mandatory.

## Post-check evidence ledger

Actual B product SHA, CI/review/ship/final-head and omitted-input consumers become available only later. Record them in the external phaseB-artifacts evidence-ledger.json (or root's finalized external successor), binding exact wrapper/action hashes, mode, public URLs, downloaded archive/sidecar/binary hashes, actual OS/architecture and step exits. Do not edit this artifact or other execution-digest inputs after check to insert future observations. An unchanged shell/process harness result must retain that exact classification; never call it hosted composite provenance unless an actual hosted uses step ran.

Both omitted-input receipts must name the exact commit selected as the floating-v6 target, not merely a nearby product/archive/merge head. If that target identity changes, qualify the actual chosen wrapper again; identical-looking version text or a historical prototype result is not the required exact-target consumer evidence.


## Recorded actual activation receipts

Stable publication/assets validation is complete: /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/validated-stable-publication.json (SHA256 `52fe7eb583d1b4c50e1f516f54f78a5b08019157b693cfd910125e8bfaeefad8`). The actual stable release `v6.1.1`, immutable RC and final tag resolve to candidate `e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa`; five public archives and five sidecars were matched to actual promotion artifacts/provenance. Dual-OS immutable qualification remains run `38752343005`.

Both explicit consumers are complete in /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/explicit-consumers.json (SHA256 `50b0f135ea07c124e56989ff9eaba7010e3fba2176322607cecbd22ef0f42897`). They executed the frozen immutable-A Action shell steps against real public assets: native macOS/aarch64 and cached Docker Linux/aarch64. This is literal Action-shell/process evidence, not full hosted GitHub composite-engine proof. No mirror or comments were used.

| Platform | Actual child receipt and hash | Stage exits |
|----------|-------------------------------|-------------|
| macos/aarch64 | /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/macos-explicit-_3ek3u28/receipt.json (SHA256 `b0321aa51232236b24ca3b9918b1fb1c8c23301c388b76ebff3bab6274602993`); wrapper receipt `152517-CODEX-958d07` | download/version/positive/coverage `0`; actual Action negative and negative JSON `1` |
| linux/aarch64 | /Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/linux-explicit-89x5vmnz/receipt.json (SHA256 `ee0e825f14c9b882a49f1178aad8739d36de8fa264c38c1fc8963cde3d084629`); wrapper receipt `152538-CODEX-dc6b6b` | download/version/positive/coverage `0`; actual Action negative and negative JSON `1` |

Both children bind immutable wrapper/action SHA256 `5fefb55e785da35940eb8660884de1a6e057777a15f63dff9156b40896fc0b72`, download-step SHA256 `ce14d6db30f132ed1a4075c4deea4c0174c0339a6d6908305e5c8b99811ad61c` and check-step SHA256 `68121d7cab4c5108b86d585f9ea2fce0eeccc5a9ec9beb4ca597b2b993d326f7`. They supplied explicit `version: 6.1.1` while observing the unchanged embedded default `6.1.0`. Each receipt records exact public URLs, archive/sidecar/downloaded-binary hashes, logs and all real exits. These actual activation results do not qualify the new B source tree or its future omitted-input consumers.

## Completed actual Phase B local qualification

The actual applied nine-file source is independently reviewed and fully qualified before this native product check. Permanent requirements and all frozen non-.specsync inputs remain unchanged. This completion records only local pre-check work, not later hosted CI or omitted-input consumer outcomes.

Input source freeze: `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-freeze.json`, SHA256 `f6fb30d6b774baf6a96ba4f37adbb8666fc3006485c3b5ee05d15e0520c53c83`.
Independent review: `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-independent-review.json`, SHA256 `a4d24929bd774c717dd3c11489d5d4853ca012b931bbe875696696a515dbeed1`; zero blocking findings, exact nine applied hashes.
Actual full Rust result records (passed/failed/ignored/measured/filtered): `[["2582", "0", "0", "0", "0"], ["462", "0", "0", "0", "0"]]`. Actual Python control summaries include64 and29 successful tests.

- Command argv `["fledge", "lanes", "run", "verify"]`; exit `0`; log `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-153300-3b1cc0-0.log`; SHA256 `225268f4e147068e8628eaee905f337fc23cc86b13b5597bafae80f513b2604d`.
- Command argv `["python3", ".github/scripts/validate-release-version.py"]`; exit `0`; log `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-153300-3b1cc0-1.log`; SHA256 `469db1755b320856c4524c0a80b195d29a4faa1f37d332e514dadbf4fe7fee22`.
- Command argv `["/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/consumer-harness/runs/macos-explicit-_3ek3u28/runner-temp/specsync/specsync", "check", "--strict", "--require-coverage", "100", "--force", "--json"]`; exit `0`; log `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-153300-3b1cc0-2.log`; SHA256 `ce0e4a2e3b8395cc6f2817bdb0d490604ed8c67ef9e0010a0a6a39a70b0eab30`.
- Command argv `["target/release/specsync", "check", "--strict", "--require-coverage", "100", "--force", "--json"]`; exit `0`; log `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-153300-3b1cc0-3.log`; SHA256 `ce0e4a2e3b8395cc6f2817bdb0d490604ed8c67ef9e0010a0a6a39a70b0eab30`.
- Command argv `["fledge", "run", "audit", "--", "--db", "/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/current-advisory-db", "--no-fetch", "--json"]`; exit `0`; log `/Users/leif/Development/_CorvidLabs/_plans/2026-10-14-specsync-6.1.1-release/action-delivery/phaseB-qualification-153300-3b1cc0-4.log`; SHA256 `abcc6e95fa61585438294d9d2b687a9b747ffca9634a7e042daa393ac5d166f1`.

Genuine conditional0xLeif acceptance and observed stable/explicit-consumer activation are recorded above. The native check below is a distinct next gate. Pre-push/Trust/Attest, exact-product CI, genuine scoped review immediately followed by ship, archive/final CI, normal merge and both exact-target omitted-input B consumers remain required before any separately authorized floatingv6 update. No future gate is marked passed.

## Where these lessons go

- `specs/github/github.3md`
