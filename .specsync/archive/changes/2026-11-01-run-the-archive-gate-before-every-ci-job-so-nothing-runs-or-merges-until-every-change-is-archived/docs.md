---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: docs
---

# Docs

- `site/src/content/docs/integrations/github-action.md`: the `require-archived` and
  `archive-gate-only` inputs, an "Archive Gate" section with a job graph where every job
  `needs: archive`, why the required check must be an aggregate `always()` job (GitHub treats a
  skipped job as passing), and that `@v6.1.1` ignores the inputs.
- `site/src/content/examples/ci-gate.mdx`: input list.
- `AGENTS.md`, `docs/ADOPTING.md`, `docs/HLD.md`, `docs/ci-confidence.md`,
  `site/src/content/docs/workflow.md`, `site/src/content/docs/cli.md` and the installed agent
  skills/commands: the happy path is `change check --commit` → `fledge lanes run verify` →
  `change review` → `change ship` locally, then push the archived tip; CI stops at Archive for
  an unarchived tip.
- `CHANGELOG.md` Unreleased entry.
- Lessons: `specs/github/github.3md`, `specs/cmd_change/cmd_change.3md`, `specs/agents/agents.3md`.
