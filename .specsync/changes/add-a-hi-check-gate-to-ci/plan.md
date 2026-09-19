---
change: add-a-hi-check-gate-to-ci
artifact: plan
---

# Plan

Two files change: `.github/workflows/ci.yml` and `Cargo.lock`.

1. **Add a `hi-check` job.** Mirrors the shape of the existing `audit` job: checkout,
   `dtolnay/rust-toolchain@1.89.0`, `cargo install human-intent --version 0.5.0 --locked`,
   then `hi check`. Gated on `classify` with `if: needs.classify.outputs.full == 'true'`,
   so doc-only and archive-only runs skip it like every other full-run gate.

2. **Add `hi/**` to the `push` and `pull_request` path filters.** Without this, a pull
   request touching only `hi/*.md` never triggers `ci.yml` at all, and because
   `Required CI gate` is a required status check that pull request would wait forever on a
   status nobody reports. The workflow's own comment states the rule this follows: every
   mergeable path must reach the gate. `INTENT.md` is already covered by the existing
   `*.md` entry.

3. **Add `hi-check` to `implementation-gate.needs` and to the corvid-pet job's `needs`
   and `CHECK_*` table.** Otherwise the job runs but cannot block a merge, and a
   structurally invalid intent file can still receive a passing scoped-review comment.
   A `skipped` result counts as a pass on `implementation-gate`, so archive-only and
   review-only pull requests are unaffected.

4. **Bump rustls in `Cargo.lock` to >=0.23.45** (RUSTSEC-2026-0285). The advisory landed
   after this branch opened; without the bump, `cargo audit` stays red on every full run.

No source, schema or spec change. `specs/github/` already owns the workflow files, so this
is declared `--no-spec-change`.
