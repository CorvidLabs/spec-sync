# Plan

Remove the orphaned test method and leave a comment where it stood recording what it asserted,
why the subject is gone, and which change removed it. A future reader finding a gap in the
`WorkflowSourceContractTests` sequence should not have to reconstruct that from git history.

## Enumerate before editing

`test-validate-release-candidate.py` contains twenty `release.index(...)` / `release.count(...)`
anchors into `release.yml`. Rather than fix the one CI happened to report, resolve every literal
anchor against the post-CHG-0150 file and list the orphans:

    MISSING ANCHOR: '          export CHECKS_FILE="$checks_file"'
    (1 of 20)

Exactly one. The other nineteen — the `qualify` job topology, the trigger block, concurrency,
the `resolve` job, rulesets, platform evidence upload, `promote`, `authorize-release`,
`record-qualification`, the release job — all still resolve, so no other test in the file is
silently asserting against text that moved.

Two nearby assertions deserve an explicit check because CHG-0150 touched the `resolve` job:
`assertNotIn('mode="release"', resolve_job)` and
`assertNotIn("needs.resolve.outputs.mode == 'release'", release)`. CHG-0150 added
`mode="dry-run"`, not `mode="release"`, so both still hold.

## Not in scope

The nineteen surviving anchors, and the file's other 48 tests. This change removes one method
and adds a comment.
