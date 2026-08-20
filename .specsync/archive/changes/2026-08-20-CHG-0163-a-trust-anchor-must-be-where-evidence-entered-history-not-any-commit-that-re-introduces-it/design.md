# Design

One rule: **for an archived change, an acceptance anchor must be the earliest reachable commit
that introduced this change's package** — identified by the ID inside the committed `state.json`,
not by directory name. The active-workspace stages and the working-tree fallback are admitted
only for commits that precede that introduction, and not at all once history holds the package.

Bounding all four stages rather than only the archived one is what catches the forged
reopen/re-archive, which never touches the archive path.

## Three details that are load-bearing

Each was a defect in a rejected candidate, found by attacking them rather than by reasoning:

- **`git_repo_prefix`, not a bare `ARCHIVE_PATH`.** Comparing a project-relative prefix against
  Git's repo-relative output makes the whole fix a silent no-op wherever the project is not at
  the repository root. Reproduced on a real nested repository: the corrected query resolves the
  original add; the naive one matches nothing at all.
- **`--no-renames`, not `--follow`.** `diff.renames` has defaulted on since Git 2.9, so a
  `git mv` is reported `R100` and vanishes from `--diff-filter=A`. A `--follow`-based fix would
  look closed while resting on a similarity heuristic the attacker controls. `--no-renames`
  forces every first appearance of a path to surface as an addition, so the *ordering* rule
  decides rather than Git's guess.
- **Identity from the committed `state.json`**, matching `find_change_dir`, which already
  resolves an archived package by parsing every `state.json` under the archive root. The
  directory name is not part of a package's identity anywhere else in this code base, so it must
  not be part of the trust decision.

## The generation term

`admissible_archive_introductions` keeps introductions that no strictly earlier introduction of
the same change supersedes, where "earlier" is qualified by `approvals.json`'s `reopenings.len()`.

Plain minimality would be wrong going forward: `reopen_change` accepts an `Archived` record, so a
legitimate reopen-then-re-finalize puts a *second* introduction in history and "earliest always
wins" would refuse it. A genuine reopen increments the reopen ledger; a copied or relocated
package does not.

Measured: 19 of 117 carry a non-empty reopen ledger, and exactly one change ID has more than one
introduction — a legacy package that never reaches this code. **The term is dormant today** and
exists so the fix does not break the reopen lifecycle later. Its soundness rests on `reopenings`
being tamper-evident, which is the unsigned-evidence half of #660 and is deliberately a separate
change.

## Regression, measured not estimated

Sampled one representative per risk class against a 161-row baseline captured before the fix:

| class | archives | expected | actual |
|---|---|---|---|
| legacy baseline | 44 | authenticated-history | authenticated-history |
| stage A+B | 19 | authenticated-history | authenticated-history |
| **stage B only** | **90** | authenticated-history | authenticated-history |
| pre-existing corrupt | 7 | corrupt-history | corrupt-history |

The stage-B-only class is the one that matters: it is the majority, and it is what breaks if the
bound is too tight. The corrupt class is a real control — those seven fail *downstream* of the
anchor logic, so they confirm the fix did not paper over unrelated failures by authenticating
more than it should.
