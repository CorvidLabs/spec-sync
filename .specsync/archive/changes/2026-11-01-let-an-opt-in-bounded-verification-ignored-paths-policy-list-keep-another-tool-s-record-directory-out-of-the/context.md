---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: context
---

# Context

Issue #825, reported downstream as CorvidLabs/chough#12 (found during CorvidLabs/raven#644).

A workflow tool next to SpecSync, here Chough, writes its records under its own directory
(`.chough/`). It writes them between `change check` and `change review`, because that is when the
human review is prepared. Any such file makes the recorded verification stale, and
`change review` refuses with `verification project-input digest is stale`. No SpecSync setting
excludes the directory:

- SDD `ignored_paths` has no effect on the digest.
- `config.toml` `exclude_dirs` and `exclude_patterns` have no effect on the digest.
- Git ignore rules hide the files only while they are untracked. The records are meant to be
  committed and shared.

The options left to the user are all bad: stash the records before every review, keep them
untracked, or re-run `change check` after every write.

## Reproduction on current main

The issue's `base.sh` and `repro1.sh` were run unchanged, except that the holding directory moved
from `/tmp` to a disposable scratch directory. The fixture was Chough's `examples/shipping` at
CorvidLabs/chough@2756dc2, with approvals and reviews by actors labelled `SIMULATED-...` in a
throwaway repository. No human decision is recorded there.

The binary was built from origin/main a80748ff (`cargo build --release`; it reports
`specsync 6.1.1`). For this revision the repro was run again, twice in parallel in fresh
directories with the fixture copied from a local CorvidLabs/chough checkout at 2756dc2: once with
that build and once with the installed `specsync 6.1.1`. Both logs were kept and compared with
`diff`. The only differing lines were the two commit ids in step G (`7edcce0`/`e20ac41` for the
installed binary, `7884e1f`/`d897cd2` for the a80748ff build). The output of the a80748ff build:

```text
== A. verified, clean tree
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== B. one untracked file under .chough/ (another tool's record)
next_action: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== C. same file hidden through .git/info/exclude
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== D. add .chough/ to sdd.json ignored_paths, commit, re-verify, then restore the record
Committed the verification evidence.
Verified, committed, and consistent with the committed tree.
next_action after re-verify (no .chough/ yet): run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
next_action with .chough/evidence/obs.json present: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== E. config.toml exclude_dirs = [".chough"] also does not help
3:exclude_dirs = [".chough"]
4:exclude_patterns = [".chough/**"]
Verified, committed, and consistent with the committed tree.
next_action with .chough/evidence/obs.json present: run `specsync change check free-shipping-at-the-threshold`
== F. commit the record instead
next_action after committing .chough/: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== G. a tracked edit under .chough/ during change check --commit
check exit=0
7884e1f chore(lifecycle): record free-shipping-at-the-threshold verification
d897cd2 chore(lifecycle): materialize free-shipping-at-the-threshold
.chough/evidence/obs.json
```

One more probe ran in the same repository. With an unknown `"verification_ignored_paths": [".chough/"]`
key added to `sdd.json`, `change status` still parsed the policy and reported
`run specsync change check ...`. Today's binaries therefore ignore the key silently, which matters
for compatibility (see `design.md`).

## Cause

1. `project_input_digest_uncached` (`src/change/collect_requirement_ids.rs:3326`) hashes every
   discovered project path except those for which `project_input_is_volatile` (`:3777`) is true.
   That predicate is a fixed, built-in list: `.git/`, `target/`, `node_modules/`, the site build
   outputs, `.specsync/changes/`, `.specsync/archive/`, `.specsync/hashes.json`, the lock and the
   transaction journal. No policy input reaches it.
2. Discovery is `git ls-files --cached --others --exclude-standard` in `git_project_paths`
   (`:3356`), filtered by the matching hard-coded byte predicate `git_path_bytes_are_volatile`
   (`:3453`). Untracked ignored files are left out, which is why step C helps. Tracked files are
   listed whatever the ignore rules say, which is why steps F and G do not help. Outside Git,
   `strict_walk_project_paths` (`:3501`) applies the same fixed predicate.
3. `SddPolicy` (`src/change/sdd_version.rs:672`) has no field for this. `ignored_paths` is read
   only by the coverage predicate `path_is_meaningful_with_specs`
   (`src/change/validate_loaded_change.rs:1113`) and by the coverage error note
   (`uncovered_paths_error`, `:829`). Nothing under `src/change/` reads `config.toml`
   `exclude_dirs` or `exclude_patterns`. They belong to spec validation (`src/config.rs`,
   `src/validator/`), which explains steps D and E.
4. One digest is compared everywhere, so a single stray file stales every gate:
   - verification records it (`verify_change_locked`, `src/change/sdd_version.rs:3100`);
   - status and review compare against it (`verification_is_current_checked`,
     `src/change/validate_loaded_change.rs:3424`, and scoped-review currency,
     `src/change/historical_definition_digest_matches.rs:1808`);
   - acceptance compares against it (`:2058`), and finalization requires the review, verification
     and finalization digests to be equal (`:1621`, `:1673`);
   - `change audit` compares against it (`:4057`).
5. There is a second binding that the digest does not cover. After a review, the descendant walk
   `review_commit_walk` (`src/change/historical_definition_digest_matches.rs:1953`) stales the
   review for any commit that touches a path outside this change's own lifecycle records
   (`:2019`-`:2034`), using repository-relative paths (`git_repo_relative_path`, `:1972`). None of
   the repro steps commits after the review, so this draft leaves the walk unchanged: a Chough
   record committed after the review still stales it, as the AGENTS.md "review and ship are one
   step" rule already says. Relaxing it is a possible follow-up (`tasks.md`).
6. Step G is a separate behaviour: `change check --commit` (`run_checked_commit`,
   `src/commands/change.rs:4119`) stages tracked edits, so a tracked `.chough/` edit lands in the
   `chore(lifecycle): materialize ...` commit. REQ-change-102 makes the change domain the only
   owner of that staging decision.

## Lessons and constraints that apply

- The REQ-change-103 lesson in `specs/change/change.3md`: the digest is
  `specsync.project-input-digest.v4`, it names clean tracked files by object id, and unsupported
  filters fail closed. An empty list must leave it byte-identical.
- Issue #676: an `ignored_paths` override that was disclosed only at audit, "three verbs too
  late". An invalid entry in the new list has to be refused when the policy is loaded, by every
  lifecycle verb.
- `ignored_paths` means "not meaningful for SDD coverage". The issue proposes a separate key
  because #676 shows how easily two meanings get mixed. This draft keeps them separate.
- REQ-change-109: no change source file may exceed 5,000 lines. The five `src/change/*.rs` parts
  are at 4,150-4,434 lines, so the new policy code goes into a new include part.

## Human intent

This draft links existing criteria and captures none:

- SHIP-2.b: editing the code after a review makes that review stale. Committing anything after a
  review still stales it, because the descendant walk is unchanged. One narrowing remains and is
  open for the human (open question 2 in `design.md`): an *uncommitted* edit under a listed prefix
  after the review leaves the review current. Entries cannot overlap sources, specs or meaningful
  paths, so this draft reads such records as not "the code", but that reading is not settled.
- SHIP-3.a: finalizing needs verification and review that are both still current.
- SHIP-1.d: the evidence matches the tree it describes, except for the prefixes the human named.
- CHANGE-7: one next action. An invalid list becomes the one next action in `change status`.

Possible wording for a new criterion, for the human to accept, reword or reject. It is not
captured, and no `hi` command has been run:

> Records another tool keeps beside my project, in a folder I named, do not make my change's
> verification go stale before it is reviewed.

If the human decides that SHIP-2.b should not be narrowed for uncommitted edits either, SHIP-2.b
needs no change and the design must bind the review to an unfiltered digest instead. If the human
accepts the narrowing, SHIP-2.b's wording may need a human-approved adjustment, for example to say
"the code or anything SpecSync verifies". Neither is proposed as captured text here.
