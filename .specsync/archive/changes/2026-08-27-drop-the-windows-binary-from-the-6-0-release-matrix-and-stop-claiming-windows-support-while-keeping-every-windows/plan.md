---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: plan
---

# Plan

The work splits into three groups. The order matters only within group 1, where the
artifact set and the gate that enforces it must move together or the release lane breaks.

## 1. Stop producing the Windows executable

1. `.github/workflows/rc-assets.yml` — remove the `x86_64-pc-windows-msvc` matrix entry,
   the `Package (Windows)` pwsh step, the now-unreachable `if: runner.os != 'Windows'`
   guard on `Package (Unix)`, and the `.zip` / `.zip.sha256` upload globs. Correct the
   header comment that says the lane builds "six targets" and spends "six build jobs".
2. `.github/workflows/release.yml` — the same four removals in the `build` job, plus the
   `suffix = ".zip" if RUNNER_OS == "Windows" else ".tar.gz"` fork in `Verify packaged
   checksum`, which now has one branch.
3. `.github/workflows/release.yml` — remove `artifacts/**/*.zip` from the `Create release`
   file list. This is load-bearing: the step sets `fail_on_unmatched_files: true`, so a
   glob that no job can satisfy any more turns every future release red.
4. `.github/scripts/validate-release-candidate.py` — drop the
   `specsync-windows-x86_64.exe` entry from `EXPECTED_ARTIFACT_ARCHIVES`. This map is
   consumed by `require_exact_entries`, which fails on a missing *and* on an unexpected
   directory, so it is the gate that must move in lockstep with step 1.
5. `.github/scripts/test-validate-release-candidate.py` — one test names
   `specsync-windows-x86_64.exe` literally; repoint it at an artifact that still exists.
   Every other test derives its fixtures from the two module constants and adapts on its own.

## 2. Stop claiming a Windows executable exists

6. `action.yml` — a `Windows` runner currently selects `specsync-windows-x86_64.exe.zip`.
   After step 1 that asset 404s, so the branch must not survive as a download path. Replace
   the OS-detection arm with an explicit refusal naming WSL, and collapse the `.zip` vs
   `.tar.gz` fork (and its hand-rolled checksum comparison, which existed only for the zip).
7. `README.md` — "Download macOS, Linux, or Windows binaries".
8. `site/src/content/docs/integrations/github-action.md` — the `Available Binaries` table
   row, the `supported Linux, macOS, and Windows smoke tests` sentence, and the
   `Multi-Platform Matrix` example that tells readers to run the action on `windows-latest`.
9. `site/src/content/docs/quickstart.md` — "Download the binary for your platform".
10. `site/src/content/docs/comparisons/adversarial-proof.md` — "CI runs the same
    deterministic binary on Linux, macOS, Windows".

## 3. Keep the correctness guarantees from narrowing with the shipped set

This is the part that is easy to miss and expensive to get wrong. Several requirements
express a Windows-content guarantee *in terms of the platforms we ship binaries for*.
Shrinking the shipped set silently shrinks those requirements — the opposite of the intent.
Each is rebound to the platforms a repository may be **checked out on**:

11. `specs/change/requirements.md` — REQ-change-083 ("a legal directory component on every
    platform SpecSync ships a binary for", and "the shortest maximum path length of any
    supported platform", which is the `MAX_PATH` 260 constraint) and REQ-change-084 ("a name
    a supported platform reserves").
12. `specs/commands/requirements.md` — REQ-commands-013 ("a directory some supported
    platform cannot open"). REQ-commands-004 already says "on every host" and is left alone.
13. `specs/commands/commands.spec.md` — the `is_reserved_module_name` description, "cannot
    be a directory component on a supported platform".
14. `src/commands/mod.rs` — the same phrase in the doc comment above that function.
15. `specs/view/view.spec.md` Invariant 8 and `specs/view/context.md` — both describe the
    shipped CRLF `view` defect as happening "on the one platform that ships a binary".
    That sentence stops being true; it must name Windows and record that it is no longer shipped.

Then the ordinary platform-set corrections:

16. `specs/github/requirements.md` REQ-github-002, `specs/github/testing.md` reviewer
    checklist, `specs/github/tasks.md` open task, `specs/github/context.md` — all four say
    the floating Action ref advances only after consumers pass on Linux, macOS **and
    Windows**. After step 6 an Action consumer cannot pass on Windows at all.
17. `specs/cli/requirements.md` and `specs/cmd_migrate/requirements.md` `## Constraints` —
    "Must work on Linux, macOS, and Windows". Rewritten to separate the shipped set from the
    portability mandate, so the constraint that produced the `#[cfg(windows)]` code survives.
18. `CHANGELOG.md` — a `### Removed` entry under `[Unreleased]`.

## Explicitly not in this plan

- The `qualify` lane in `release.yml` (`platform: windows` / `windows-latest`) and
  `REQUIRED_PLATFORMS` in the validator. See `context.md` — that lane is what compiles and
  runs the `#[cfg(windows)]` code this change is protecting, and dropping it is the
  anti-pattern `docs/ci-confidence.md` names.
- `linux-aarch64`, which the owner did not approve. See `research.md`.
