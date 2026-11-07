---
change: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
artifact: design
---

# Design

**attest.** `if: ${{ always() && needs.ci-gate.result == 'success' && github.event_name == 'push' && github.ref == 'refs/heads/main' }}`.
- `always()` stops the implicit transitive `success()`.
- The explicit `needs.ci-gate.result == 'success'` keeps the attestation behind the required gate, which itself needs every gate job.
- `needs` stays `[ci-gate]`, and the step is unchanged and best-effort.

**Summary rows.** Each `CHECK_*` value becomes `<label>=<selected>=<result>`. `<selected>` is `true`, or the job's own `if:` less a leading `always() &&`, as a `${{ }}` expression. It is the same selection `implementation-gate`'s GATES rows carry.
- Two rows are added: `CHECK_PREFLIGHT` ("Lifecycle preflight") and `CHECK_LIFECYCLE` ("Lifecycle gate"). Both are always selected.
- The script splits each value into label, selection and result:
  - `success` → ✅ Passed
  - `false:skipped` → ⏭️ Not selected
  - `true:skipped` → ⛔ Blocked (a job it needs did not succeed), and the summary fails
  - anything else → ❌ \<result\>, and the summary fails
- Labels contain no `=`.
- `REPORT_SPEC`, the details block and the corvid-pet Action inputs are unchanged.

**Test.** `.github/scripts/test-ci-attest-and-summary.py` imports `test-required-ci-gate.py` as a module, as `test-ci-trigger-paths.py` does.
- **attest:** `GATE.simulate` over the model's own `SCENARIOS`.
- **Summary:** it parses the `CHECK_*` rows, holds each selection to `GATE.selection_expression` of its job, and requires one row per waited-on job.
- **Running the real step:** it runs the step's real script under `bash -e` with `GITHUB_OUTPUT` set, over env values interpolated from a simulated run. That covers a green full pull request, a failed lifecycle gate, a failed preflight, and hand-built deselected and blocked rows.
- It runs as fledge task `ci-attest-and-summary-test` in the `ci`, `verify` and `repo` lanes, and as a `validate-action` step in `ci.yml`.
