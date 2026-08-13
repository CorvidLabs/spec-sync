---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: research
---

# Research

Root-cause findings for each defect, and the second-order bugs each investigation
surfaced.

## 1. `init` leaves an unsatisfiable coverage gate

`path_is_meaningful_with_specs` consults `is_protected_sdd_path` and returns `true`
before the ignore filter is applied. That protection is correct in general — a hand
edit to `.specsync/sdd.json` is exactly the delivery a change workspace should have
to declare — but the protected list is *precisely* the set of files `init` writes.

The failure is therefore not a misconfiguration a user can escape. Every possible
first commit after `specsync init` is uncovered meaningful delivery, and no change
workspace can cover it, because the files predate the existence of any workspace.

**Second bug found in the same code path:** `recorded_diff_base` falls back to
`HEAD~1...HEAD`. In a repository with one commit — the overwhelmingly common state
immediately after `git init` — `HEAD~1` does not resolve, so the gate errors out on
a repository shape the quick-start guide produces.

**Design constraint discovered:** the exemption cannot be pinned to the *bytes* of
`sdd.json`. `init` writes an empty `verification_commands` list whenever it cannot
detect a test command, and prints instructions telling the author to fill it in.
Pinning bytes would revoke the bootstrap for doing exactly what the tool just asked
for. The digest therefore pins the **enforcement surface** — `enabled`,
`require_change_for_meaningful_files`, `meaningful_paths`, `ignored_paths`, custom
artifacts, principles — and clears `verification_commands` before hashing. This is a
judgment call and is called out as such in the design.

## 2. `scaffold` output fails the effective-contract gate

The stub-section warning is emitted by `validator::validate_spec` with fixed wording.
`WarningCategory::classify` matches `requirements` before it would reach the stub
case, so classification cannot be reused to recognize these; the section name has to
be parsed from the warning text. That coupling is real and is documented at the
constant, in the same manner `SCAFFOLD_BOILERPLATE_PREFIXES` is coupled to the
scaffold text it detects.

The correct exemption key is **authorship, not content**. A generated section that no
active change has touched is not evidence of anything and should not gate. A section
an active change authored and then emptied is a regression and must stay fatal. The
implementation routes the exemption through `IgnoreRules` so ignore configuration is
honored consistently rather than re-derived.

## 3. Directory in `files:` — #472, worse than filed

Filed as Kotlin-specific. It is language-independent.

Two independent code paths reach a `files:` entry, and both mishandled a directory:

| Path | Before | After |
|---|---|---|
| Ambient filesystem validation | `full_path.exists()` is true and `source_within_root` is true, so no branch fires; zero exports extracted, Public API comparison vacuously passes | `Source file … is a directory` error with an expansion fix |
| Snapshot validation (`issues.rs`) | `!metadata.is_file()` returned `SourceSnapshot::Rejected`, reported to the user as an out-of-root **security escape** | New `SourceSnapshot::Directory` variant reports the real cause |

The snapshot path was therefore not silent, but it was actively misleading: it told
the author their path escaped the project root when the path was confined and merely
the wrong shape. Both paths now agree on the diagnosis.

**Reuse discovered:** `generate` and `scaffold` already expand a directory listed
under `[modules."x"] files` into the source files beneath it. Making the validator's
suggested fix reuse that same expansion — `generator::find_module_source_files` —
means the remedy the error prints is byte-identical to what generation would have
written, instead of a second, subtly different notion of "the files under here".

**Constraint:** snapshot callers validate retained bytes only and must not enumerate
the ambient filesystem. They receive the shape guidance without the file list.
