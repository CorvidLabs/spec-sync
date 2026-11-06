---
change: teach-archive-first-closeout-in-the-readme-and-run-its-workflow-example-as-a-test-881-d5
artifact: design
---

# Design

## README

The closeout block keeps every command it has today (`check --commit`, `review`, `finalize`) and the actors. Only the prose comments move:

```bash
specsync change check add-passkeys --commit
# run the local verify lane; CI does not test a tip whose change is still open
# after the human implementation review of the package, diff, delta and evidence:
specsync change review add-passkeys --reviewer "Ada"
specsync change finalize add-passkeys
# no commit between review and finalize; finalize commits the archive move
# then push the archive commit, wait for CI on that tip, and merge
```

The paragraph under the block gains Chough's two sentences: the Archive gate refuses while a change is open, so finish locally before waiting on CI, and a push before the review only shows the work early.

## Test: `tests/integration/documented_workflows.rs`

`readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci` reads the first `bash` block under `## Verified change workflow` from README.md at test time and splits it into commands (joining `\` continuations) and `#` prose steps.

- Every command runs exactly as written through `bash -e -o pipefail -c`, with the built `specsync` first on `PATH`, in a fresh git project. The project holds the README's `auth` module (`src/auth.rs`, one export, `REQ-auth-001`), created by `specsync init` and `specsync change adopt`. `SPECSYNC_*`, `GITHUB_EVENT_NAME` and `GITHUB_BASE_REF` are removed, as the `specsync()` helper does.
- A prose step that tells the reader to answer the interview and complete the artifacts is done the way a reader would do it. Each remaining question gets the answer the interview recommends. Every selected artifact gets real content: the plan has its required sections, tasks are all checked, and testing has a `## Requirement evidence` row. The module delta is written too. A prose step starting "implement" edits `src/auth.rs` without changing its export and commits it. Other prose needs no action; if the reader really had to do something there, the next command fails.
- A prose step that names CI or checks and says wait or observe runs the real Archive gate (`.github/scripts/archive-readiness.sh`), and the gate must pass there. That is the D5 assertion.
- After every step while the change workspace exists, the gate must refuse. Each distinct command the refusal names (`specsync change status <id>`, and `specsync change ship <id>` after the review) runs in a copy of the project. `status` must print a `Next:` line, and after `ship` the gate must pass in the copy. The test requires the `ship` recovery to have been exercised.
- The block must end with the change archived and the gate passing.

The actor and reviewer labels are fixture simulations, not human decisions.

The test is in `cargo test` (`tests/integration.rs`), so it runs in CI's `test` job and in `fledge lanes run verify`. README.md already classifies as a full-lane path (`classify-ci-paths.sh`, the `*` fallback), so a README-only edit reruns it.

The harness (block extraction, steps, `Project`) is written so the landing-page test in #883 can reuse it.
