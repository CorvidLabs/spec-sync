# 6.0.0 confidence checklist

Fixed before any drill ran (Phase 0). Later discoveries append new claims; existing claims are
never edited. Weight: P1 = 3, P2 = 1. A claim passes only when two independent verifiers pass it
with concrete evidence (command, output, SHA) and a refuter fails to refute it.
Score = passed weight / total weight. Target ≥ 0.95 with zero open P1.

| # | W | Claim |
|---|---|---|
| C01 | P1 | Fresh Rust project: `init` → `check` exits 0 with 0 warnings on a clean tree; an added undocumented export warns; `--strict` exits 1. |
| C02 | P1 | Fresh TypeScript project: same as C01. |
| C03 | P1 | Fresh Python project: same as C01. |
| C04 | P2 | Fresh Swift project: same as C01. |
| C05 | P2 | Fresh Go project: same as C01. |
| C06 | P1 | Fresh git project (Rust): `change adopt` → `change new` → `answer` → artifacts → `approve` → implement → `check --commit` → `review` → `ship` ends with the package under `.specsync/archive/changes/` and `change list` empty, following only public docs. |
| C07 | P1 | Same lifecycle as C06 in a TypeScript project. |
| C08 | P2 | Same lifecycle as C06 in a Python project. |
| C09 | P2 | Lifecycle in a shallow clone (`--depth 1`) either completes or fails with a message that names the shallow clone / `fetch-depth: 0` remedy. |
| C10 | P2 | Lifecycle in a CRLF checkout (`core.autocrlf=true`) completes; digests are stable across a re-checkout. |
| C11 | P2 | `check` in a monorepo subdirectory with `--root` behaves as in the project root. |
| C12 | P2 | Non-git tree: `init`, `check`, `coverage`, `score` work; lifecycle verbs refuse with a clear message rather than panicking. |
| C13 | P1 | `--json` output of `init`, `check`, `coverage`, `score`, `change new`, `change status`, `change list` is parseable JSON with nothing else on stdout, on a clean tree. |
| C14 | P1 | Upgrade: a 5.2.0-initialized project with one in-flight workflow-v1 change, following `MIGRATION.md` literally, lands the v1 change, runs `change adopt`, and archives a v2 change. |
| C15 | P1 | A v1 change merged after the v2 cutoff is refused by adopt/list/status with the documented message (#674). |
| C16 | P1 | Enforcement default: bare `check` exits 1 on a validation error; `--enforcement warn` exits 0; config `enforcement = "warn"` exits 0. |
| C17 | P1 | GitHub Action from this tree on `CorvidLabs/spec-sync-sandbox`, `ubuntu-latest`, `version: 6.0.0-rc.14` (or `download-base-url` mirror of the tree binary): passes on a clean tree, `strict: true` fails on drift. |
| C18 | P2 | Same as C17 on `macos-latest`. |
| C19 | P2 | Action `comment: true` posts or updates the PR comment on the sandbox. |
| C20 | P1 | README quick start executed literally by a text-only agent succeeds end to end. |
| C21 | P1 | `site/src/content/docs/quickstart.md` executed literally succeeds end to end. |
| C22 | P1 | `site/src/content/docs/workflow.md` executed literally (one change through the lifecycle) succeeds. |
| C23 | P1 | `MIGRATION.md` executed literally on a 5.2.0 project succeeds. |
| C24 | P2 | Each `examples/*/README.md` executed literally succeeds; `examples/*/run.sh` pass with the tree binary. |
| C25 | P2 | `docs/RELEASING.md` sections 1 to 3 executed literally succeed (dry run only, `v6.0.0-rc.16`). |
| C26 | P1 | `specsync <verb> --help` for every verb and `change` sub-verb matches the flags and defaults in `site/src/content/docs/cli.md`. |
| C27 | P1 | JSON shapes of `check`, `change list`, `change status`, `coverage`, `score` match their documented shape on a clean tree and on a degraded tree (array vs object variant for `change list/status` is the contract). |
| C28 | P2 | Exit codes: every documented non-zero exit condition in `cli.md` reproduces; no unexpected non-zero exit on the happy paths. |
| C29 | P2 | Unknown config key warns and exits 0; unknown `state.json` field survives a read; `workflow_version: 3` is refused with the documented message; `sdd.json` with `version: 3` is accepted. |
| C30 | P1 | Tamper: editing `approvals.json` actor after approval is either refused or documented as a boundary; the digest still binds content (changing approved scope content is refused). |
| C31 | P1 | Tamper: deleting `verification-attempts.json` before `ship` is refused, or the behaviour is recorded as a known 6.0.x defect (#656) with the exact observed outcome. |
| C32 | P1 | Tamper: `git mv` of an archive directory is reported as corrupt/refused by `change audit`, not laundered. |
| C33 | P1 | Tamper: stripping `workflow_version` from a v2 record is refused as the documented downgrade (#603). |
| C34 | P2 | Concurrency: two clones minting the same slug meet at merge with a refusal, not silent divergence; two clones editing one shared file both complete `check --commit` sequentially. |
| C35 | P1 | `check --fix`: new rows land inside the existing table under heading, bold, and no label; fenced and indented samples are untouched; a following `check --strict` passes. |
| C36 | P2 | `import --from-dir` writes a draft skeleton and `check` fails on it exactly as the docs caveat says (#416). |
| C37 | P1 | Release lane: `workflow_dispatch` `dry_run=true` against `v6.0.0-rc.16` passes `resolve` and `validate`; the 52 validator tests pass; both Bash guards require exactly two evidence records. |
| C38 | P2 | Consumer pins verified live for all four consumers. |
| C39 | P2 | `check` on this repository completes under 2 s; `change list` with 20 drafts measured and reported. |
| C40 | P1 | MCP: `initialize`, `tools/list` (5 tools; 7 with `--allow-write`), one `specsync_check` call succeed on a fresh project. |
| C41 | P1 | `fledge lanes run verify` passes on the final tree; strict spec validation at 100 percent coverage. |
| C42 | P2 | `cargo package --list` matches the documented include set; `cargo publish --dry-run --locked` succeeds offline-safe (no upload). |
