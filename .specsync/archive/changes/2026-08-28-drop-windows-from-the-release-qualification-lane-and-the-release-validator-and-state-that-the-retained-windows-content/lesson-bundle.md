# Lesson bundle — drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Drop Windows from the release qualification lane and the release validator, and state that the retained Windows content guarantees are now unverified
- **Kind**: Operations
- **Specs**: cmd_issues, github
- **Paths**: .github/workflows/release.yml, .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, src/commands/issues.rs, docs/ci-confidence.md, CHANGELOG.md
- **Acceptance**: The release qualify matrix contains only ubuntu and macos; REQUIRED_PLATFORMS is ('ubuntu','macos'); no Windows-only step remains in release.yml; the validator self-test passes 50/50; docs/ci-confidence.md and CHANGELOG.md state that the retained cfg(windows) code is now compiled and run nowhere and that those guarantees are best-effort and unverified; and the CHANGELOG paragraph that argued for keeping the lane says the argument was correct and the risk is accepted rather than resolved.

## Evidence

- Verification commit: `8be8aeea6c37156ca8731aba7c05f2e5c5bcb664`
- Base commit: `4b72b09de0e950b7a0479463dbefcac33d516cac`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

## What led here

The owner asked to drop Windows CI, after seeing that PR #734 was *adding* a Windows compile job to
ordinary CI and asking the reasonable question: didn't we already remove Windows?

We removed the **binary** (#722). We deliberately kept the **qualification lane**, and #734 was
about giving that lane earlier signal. The owner's decision reverses the retained half.

## The argument being overruled, stated because it was correct

The `### Removed` entry for #722 says the qualification lane must stay:

> It is the only place the retained `#[cfg(windows)]` code is compiled and run, and removing it
> would recreate exactly the condition that produced the `view` defect.

That is true and it is not softened here. `#[cfg(windows)]` code is now compiled and run **nowhere**.
CRLF frontmatter tolerance, reserved-name and Windows-invalid filename guards, `MAX_SLUG_BYTES` and
its `MAX_PATH` justification, junction and reparse-point rejection and path-separator handling are
all retained, still believed correct, and **unverified**. The CHANGELOG paragraph that made the
argument now says the argument was correct and the risk is accepted rather than resolved, instead of
being quietly deleted — a decision reversed silently reads later as a decision never made.

## The case for the reversal

The lane only ever ran on a tag push, so its first signal arrived at the worst possible moment. It
cost `rc.8` and `rc.9`, on a defect latent since #544 that no ubuntu-only job could see. That is the
same argument the Windows binary was dropped on — an artefact nothing exercises — applied one level
further.

## What was kept from #734, and why

The `open_specs_directory` gate fix. It is `#[cfg(test)]` rather than `#[cfg(all(test, unix))]` and
the three helpers are imported unconditionally. **This is correct independently of which platforms
CI compiles for**: a `files:` entry resolving to a directory is a spec-content error on every
platform a repository may be checked out on, and its ambient-path twin
(`validator::tests::directory_source_mapping_fails_loud_and_names_the_files_to_list`) has always
been ungated. Keeping it costs nothing and removing it would re-narrow a platform-independent
guarantee to whichever platform happens to compile it.

What was **dropped** from #734 is the new `windows-check` job in `ci.yml` — the whole point of which
was to protect the lane this change removes.

## The coupling that will bite whoever reverses this

`REQUIRED_PLATFORMS` in the validator and the `qualify` matrix in `release.yml` must move together.
Adding a platform to one without the other fails **every** candidate: the validator demands evidence
the matrix never produces. A comment at the constant says so, and the CHANGELOG entry says so.

## Ruled out

- **Deleting the retained `#[cfg(windows)]` code.** `cargo install specsync` on Windows is still a
  documented path in `README.md`, and mixed-OS teams are the case the content guarantees exist for.
  Unverified is not the same as unwanted.
- **Silently removing the paragraph that argued for the lane.** See above.
- **Keeping a Windows job in ordinary CI while dropping it from the release lane.** That is the
  inverse trade and nobody asked for it: it would pay for Windows signal on every PR to protect a
  lane that no longer exists.

## From the change's design.md

# Design

Recorded in `context.md`, `docs.md` and `testing.md` for this change: the decision (drop Windows
from the qualification lane and the release validator), the argument it overrules and why that
argument was correct, the one piece of #734 that is kept because it stands on its own merits, the
`REQUIRED_PLATFORMS`/matrix coupling that must move together, and the disclosure that the retained
`#[cfg(windows)]` guarantees are now best-effort and unverified.

No canonical spec text changes: this is CI configuration, one validator constant, a test-only `cfg`
attribute, and prose. No CLI behaviour, API surface, or output format changes — the published binary
set is unchanged; what changes is which platforms we verify, and the documentation now says so.

## From the change's testing.md

# Testing

## What is verifiable

| check | result |
|---|---|
| Validator self-test | `python3 .github/scripts/test-validate-release-candidate.py` — **50 passed** |
| `release.yml` still parses | `yaml.safe_load` OK |
| No Windows-only step survives | `grep -in windows .github/workflows/release.yml` returns only the summary line stating Windows is not qualified |
| No Windows reference survives in the validator or its test | `grep -ci windows` → 0 in the test file; the constant carries an explanatory comment |
| Rust suite unaffected | the only `src/` change is a `cfg` attribute on a test-only helper |

## A test that caught a real defect in this change

Retargeting the validator's self-test from `windows` to `macos` was done by textual replacement, and
in `test_release_refuses_missing_or_mixed_evidence` that produced a **duplicate dict key**:

    "ubuntu": {"workflow_revision": "a" * 40},
    "macos":  {"workflow_revision": "b" * 40},
    "macos":  {"workflow_revision": "a" * 40},

The second entry silently overwrote the first, so the "mixed workflow revisions" case stopped being
mixed and the test passed for the wrong reason — a green run asserting nothing. Caught because the
suite failed on `assertNotEqual(returncode, 0)` rather than because anyone read the diff.

**Honest label: CONTROL.** That test must keep failing the validator on genuinely mixed evidence. If
a future edit makes its two platform records agree, it becomes vacuous again and will pass silently.

## What cannot be verified any more, by design

Everything `#[cfg(windows)]`. No job in this repository compiles it. That is the accepted cost of
this change and is stated in `docs/ci-confidence.md` and the CHANGELOG rather than left implicit.
There is no test that can assert an absent guarantee; the disclosure is the mitigation.

## Where these lessons go

- `specs/cmd_issues/context.md`
- `specs/github/context.md`
