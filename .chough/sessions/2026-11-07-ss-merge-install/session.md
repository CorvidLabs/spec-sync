---
kind: session
id: 2026-11-07-ss-merge-install
aliases: [ss-merge-install]
objective: "G-merge-install (#871): merge --install applies the SpecSync driver to root and nested contracts when specs_dir is the repository root (MERGE-1)"
intent_scope: MERGE
modules: [cmd_merge]
stage: intent
stage_status: active
iteration: 1
callers: []
native_change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
pr: ""
criteria: [MERGE-1]
requirements: []
cases: []
design: []
files: []
observations:
  - OBS-product-test-b47ebf5a0d626500e6c08d9c
  - OBS-product-test-0677784587f48c2115ff3f2c
  - OBS-product-test-8b3cf28186053c41615bf658
  - OBS-product-test-e62f4a4b773df570bf80de09
  - OBS-product-test-f6bd8d6b1f50fef57acf10bb
bugs: [BUG-merge-install-ignores-a-root-specs-directory-871-408287a7fe659cca53621579]
blockers: []
waiting_on_human: []
run_consents: []
next: "SAFE_TO_STOP: #871 archived at e36b0e70; push of archive tip, PR #1017 merge and CI watch held for root per orc t19f"
---
# G-merge-install (#871): merge --install applies the SpecSync driver to root and nested contracts when specs_dir is the repository root (MERGE-1)

## Goal

G-merge-install (#871): merge --install applies the SpecSync driver to root and nested contracts when specs_dir is the repository root (MERGE-1)

## Where it stands

- Drafted the complete cmd_merge change and proved root_specs_dir_matches_contracts fails before the fix
- Implemented root specs normalization; focused install tests and the original installed-command scenario pass
- Human implementation review recorded by orc (t1kd exit 0) at unchanged 78f28ec4; Leif acceptance relayed by orc t19f; change shipped and archived at e36b0e70 (2026-11-08 archive), sibling/prior archive bytes preserved (renames+adds only)

## Decisions and why

- Leif (historical relay) decided "HI mapping confirmation for #871: existing MERGE-1, no new wording": "Yes to all 5 (Recommended)" (Historical Leif five-item scope approval via orc t18, 2026-11-06; native #871 approval0391a4c8 including MERGE-1 mapping, interview/wrap-up records; relayed by orc t19f); recorded by k-ss-testgaps (orc t19f relay); a relayed claim, not authenticated

## What was verified

- [At origin/main b3ca5f97, a temporary repo with .specsync/config.toml specs_dir=. ran merge --install successfully but wrote ./**/*.spec.md and ./**/requirements.md; git check-attr reported merge: unspecified for root and nested .spec.md and requirements.md paths.](evidence/2026-11-07-at-origin-main-b3ca5f97-a-temporary-repo-with-specsync-config.md)
- [On base b3ca5f97 plus only the new regression test, cargo test --bin specsync root_specs_dir_matches_contracts failed at root.spec.md with merge: unspecified; 1 test failed and 2898 were filtered out.](evidence/2026-11-07-on-base-b3ca5f97-plus-only-the-new-regression-test-cargo-test.md)
- [After the root normalization fix, cargo test --bin specsync install_tests passed 6/6 tests, including root_specs_dir_matches_contracts for both . and ./ across root and nested spec and requirements paths; 2893 tests were filtered out.](evidence/2026-11-07-after-the-root-normalization-fix-cargo-test-bin-specsync.md)
- [With the rebuilt release binary in a fresh repository using canonical .specsync/config.toml specs_dir=., merge --install emitted **/*.spec.md and **/requirements.md, and git check-attr returned merge: specsync for root.spec.md, nested/root.spec.md, requirements.md, and nested/requirements.md.](evidence/2026-11-07-with-the-rebuilt-release-binary-in-a-fresh-repository-using.md)
- [fledge lanes run verify was interrupted by the orc's SAFE-PARK instruction and exited 130. The lane did not complete, so there is no full-verify verdict.](evidence/2026-11-07-fledge-lanes-run-verify-was-interrupted-by-the-orc-s-safe-park.md)

## Open threads

## Timeline

- 2026-11-07T03:09:44Z · gate-opened · intent → intent · HI mapping confirmation for #871: existing MERGE-1, no new wording ^t-131d61
  - native_change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
  - revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
- 2026-11-07T23:09:26Z · gate-closed · intent → intent · HI mapping confirmation for #871: existing MERGE-1, no new wording ^t-351014
  - native_change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
  - revision: 6a2bde331e3cc74ea68ccbb52e755438be20c493
  - relayed_by:
    - decided_by: Leif (historical relay)
    - words: Yes to all 5 (Recommended)
    - where: Historical Leif five-item scope approval via orc t18, 2026-11-06; native #871 approval0391a4c8 including MERGE-1 mapping, interview/wrap-up records; relayed by orc t19f
    - recorded_by: k-ss-testgaps (orc t19f relay)
